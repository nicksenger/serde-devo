![serde-devo](devo.png)

# serde-devo

`serde-devo` is a utility to help minimize breaking changes when sharing types between multiple independently deployed applications.

The provided derive macro generates a new set of `Devolved*` types for which any associated enums will contain a `serde(untagged)` variant. Conversions to and from the original type are also derived as appropriate.

## Why?

Imagine we have an application which exposes this API:

```rust
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Fish {
    OneFish,
    TwoFish,
    RedFish,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct FishList {
    pub fishes: Vec<Fish>,
    pub total: usize,
}

pub fn list_fishes(limit: usize) -> FishList {
    todo!()
}
```

Years later, many clients have come to depend on this API, but we realize that we forgot about `BlueFish`! So we propose adding it to the enum and returning information about `BlueFish` as well:

```rust
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Fish {
    OneFish,
    TwoFish,
    RedFish,
    BlueFish
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct FishList {
    pub fishes: Vec<Fish>,
    pub total: usize,
}
```

Those clients which can easily update to our new types are happy with the proposed changes, but we have a big problem: several of our clients running the old code are unmanned space probes which launched just days ago, and are under strict contract not to update _any_ fish-related code during a mission!

We have no choice but to cut a new endpoint for this breaking change, and while we're at it, we use `serde(untagged)` to make sure we never get into this situation again:

```rust
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Fish {
    OneFish,
    TwoFish,
    RedFish,
    BlueFish,
    #[serde(untagged)]
    UnknownFish(serde_json::Value),
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct FishList {
    pub fishes: Vec<Fish>,
    pub total: usize,
}
```

Now clients are forced to handle the possibility of unknown fishes, future-proofing our API. But the server code, which was already starting to become unmanageable, has now ballooned in complexity unnecessarily due to the handling this `UnknownFish(_)` variant. After extensive refactoring, we eventually settle on the following:

```rust
#[derive(serde::Serialize, serde::Deserialize)]
pub enum FishOrUnknown {
    OneFish,
    TwoFish,
    RedFish,
    BlueFish,
    #[serde(untagged)]
    UnknownFish(serde_json::Value),
}

impl From<Fish> for FishOrUnknown {
    fn from(fish: Fish) -> Self {
        match fish {
            Fish::OneFish => Self::OneFish,
            Fish::TwoFish => Self::TwoFish,
            Fish::RedFish => Self::RedFish,
            Fish::BlueFish => Self::BlueFish,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct FishOrUnknownList {
    pub fishes: Vec<FishOrUnknown>,
    pub total: usize,
}

impl From<FishList> for FishOrUnknownList {
    fn from(list: FishList) -> Self {
        Self {
            total: list.total,
            fishes: list.fishes.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum Fish {
    OneFish,
    TwoFish,
    RedFish,
    BlueFish
}

impl TryFrom<FishOrUnknown> for Fish {
    type Error = String;
    fn try_from(fish: FishOrUnknown) -> Result<Self, Self::Error> {
        Ok(match fish {
            FishOrUnknown::OneFish => Self::OneFish,
            FishOrUnknown::TwoFish => Self::TwoFish,
            FishOrUnknown::RedFish => Self::RedFish,
            FishOrUnknown::BlueFish => Self::BlueFish,
            FishOrUnknown::UnknownFish(_) => {
                return Err("Received an unrecognized fish!".to_string());
            }
        })
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct FishList {
    pub fishes: Vec<Fish>,
    pub total: usize,
}

impl TryFrom<FishOrUnknownList> for FishList {
    type Error = String;
    fn try_from(list: FishOrUnknownList) -> Result<Self, Self::Error> {
        Ok(Self {
            total: list.total,
            fishes: list
                .fishes
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

pub fn list_fishes(limit: usize) -> FishOrUnknownList {
    let (total, fishes): (usize, Vec<Fish>) = todo!();
    FishOrUnknownList {
        fishes: fishes.into_iter().map(Into::into).collect(),
        total,
    }
}
```

Finally we seem to have achieved the best of both worlds. Clients receive a `FishOrUnknownList`, while our server can work with the exhaustive list of known fishes.

`serde_devo` allows us to shorten the above to:

```rust
#[derive(serde::Serialize, serde::Deserialize, serde_devo::Devolve)]
pub enum Fish {
    OneFish,
    TwoFish,
    RedFish,
    BlueFish
}

#[derive(serde::Serialize, serde::Deserialize, serde_devo::Devolve)]
pub struct FishList {
    #[devo]
    pub fishes: Vec<Fish>,
    pub total: usize,
}

pub fn list_fishes(limit: usize) -> DevolvedFishList {
    let (total, fishes): (usize, Vec<Fish>) = todo!();
    DevolvedFishList {
        fishes: fishes.into_iter().map(Into::into).collect(),
        total,
    }
}
```

The fallback type contained within the `serde(untagged)` variant can be customized with the container attribute helper:

```rust
#[derive(serde::Serialize, serde::Deserialize, serde_devo::Devolve)]
#[devo(fallback = ciborium::Value)]
pub enum Fish {
    OneFish,
    TwoFish,
    RedFish,
    BlueFish
}

#[derive(serde::Serialize, serde::Deserialize, serde_devo::Devolve)]
#[devo(fallback = ciborium::Value)]
pub struct FishList {
    #[devo]
    pub fishes: Vec<Fish>,
    pub total: usize,
}
```

## Limitations

This only works for self-describing formats like JSON / MessagePack / CBOR. It will not work for bincode / bitcode / etc.

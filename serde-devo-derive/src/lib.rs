use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned, ToTokens, TokenStreamExt};
use syn::{
    parse_macro_input, spanned::Spanned, Attribute, Data, DataEnum, DataStruct, DataUnion,
    DeriveInput, Field, Ident, Meta, Type, Variant,
};

#[proc_macro_derive(Devolve, attributes(devo))]
pub fn devolve_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let (vis, name, attrs) = (&ast.vis, &ast.ident, &ast.attrs);
    let (devo_name, devo_attr) = (format_ident!("Devolved{}", name), format_ident!("devo"));

    let mut serde_attrs = TokenStream::new();
    let warnings_mod = format_ident!("devolved_{}_warnings", name.to_string().to_lowercase());
    let devo_fallback_type = attrs.iter().find_map(|attr| match &attr.meta {
        Meta::List(list) if list.path.get_ident() == Some(&format_ident!("serde")) => {
            serde_attrs.append_all(quote! { #attr });
            None
        }
        Meta::List(list) if list.path.get_ident() == Some(&devo_attr) => {
            let mut ft = None;
            let _ = list.parse_nested_meta(|meta| {
                let Ok(ty) = meta.value().unwrap().parse::<Type>() else {
                    return Ok(());
                };
                ft = Some(ty);

                Ok(())
            });

            ft
        }
        _ => None,
    });

    #[cfg(feature = "json")]
    let fallback_type =
        { devo_fallback_type.unwrap_or(syn::parse2(quote!(serde_json::Value)).unwrap()) };

    #[cfg(not(feature = "json"))]
    let fallback_type = {
        use proc_macro2::Span;
        if let Some(ty) = devo_fallback_type {
            ty
        } else {
            let warning = syn::Error::new(
                Span::call_site(),
                "either enable the \"json\" feature or provide the `#[devo(fallback = Type)]` container attribute",
            )
            .into_compile_error();
            return quote! {
                mod #warnings_mod {
                    #warning
                }
            }
            .into();
        }
    };

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let (devo_token, (is_tuple_struct, warn, devo_body, evo_impl, devo_impl)): (
        TokenStream,
        (
            bool,
            Vec<TokenStream>,
            TokenStream,
            TokenStream,
            TokenStream,
        ),
    ) = match ast.data {
        Data::Struct(DataStruct {
            fields,
            struct_token,
            ..
        }) => (struct_token.into_token_stream(), {
            let (is_devo, is_named, tokens, evo_impl, devo_impl): (
                bool,
                bool,
                TokenStream,
                TokenStream,
                TokenStream,
            ) = fields.into_iter().enumerate().fold(
                (
                    false,
                    false,
                    TokenStream::new(),
                    TokenStream::new(),
                    TokenStream::new(),
                ),
                |(d, b, mut st, mut evo, mut dvo), (i, f)| {
                    let is_named = f.ident.is_some();
                    let (is_devo, tokens, ev, dv) = if is_named {
                        render_field(f, &devo_attr, false, &fallback_type)
                    } else {
                        render_tuple_field(f, &devo_attr, i, None, &fallback_type)
                    };
                    st.append_all(tokens);
                    evo.append_all(ev);
                    dvo.append_all(dv);
                    (is_devo || d, is_named || b, st, evo, dvo)
                },
            );

            let span = name.span();
            let mut warn = vec![];
            if !is_devo {
                warn.push(
                        syn::Error::new(
                            span,
                            "using derive(Devolve) without at least one #[devo] attribute on structs does nothing",
                        )
                        .into_compile_error(),
                    );
            }

            (
                !is_named,
                warn,
                if is_named {
                    quote! {
                        {
                            #tokens
                        }
                    }
                } else {
                    quote! {
                        (
                            #tokens
                        )
                    }
                },
                if is_named {
                    quote! {
                        {
                            Ok(#name { #evo_impl })
                        }
                    }
                } else {
                    quote! {
                        (
                            Ok(#name ( #evo_impl ))
                        )
                    }
                },
                if is_named {
                    quote! {
                        {
                            #devo_name { #devo_impl }
                        }
                    }
                } else {
                    quote! {
                        (
                            #devo_name ( #devo_impl )
                        )
                    }
                },
            )
        }),

        Data::Enum(DataEnum {
            variants,
            enum_token,
            ..
        }) => (enum_token.into_token_stream(), {
            let (is_untagged, tokens, warn, evo_impl, devo_impl): (
                bool,
                TokenStream,
                Vec<TokenStream>,
                TokenStream,
                TokenStream,
            ) = variants.into_iter().fold(
                (
                    false,
                    TokenStream::new(),
                    vec![],
                    TokenStream::new(),
                    TokenStream::new(),
                ),
                |(is_untagged, mut st, mut w, mut evo, mut dvo), variant| {
                    let (b, tokens, warn, ev, dv) =
                        render_variant(name, &devo_name, variant, &devo_attr, &fallback_type);
                    w.extend(warn);
                    st.append_all(tokens);
                    evo.append_all(dv);
                    dvo.append_all(ev);
                    (b || is_untagged, st, w, evo, dvo)
                },
            );

            (
                false,
                warn,
                if is_untagged {
                    quote! {
                        {
                            #tokens
                        }
                    }
                } else {
                    quote! {
                        {
                            #tokens
                            #[serde(untagged)]
                            UnrecognizedVariant(#fallback_type),
                        }
                    }
                },
                quote! {
                    {
                        match self {
                            #evo_impl
                            _ => {
                                Err(::serde_devo::Error::default())
                            }
                        }
                    }
                },
                quote! {
                    {
                        match self {
                            #devo_impl
                        }
                    }
                },
            )
        }),

        Data::Union(DataUnion { fields, .. }) => {
            return quote_spanned! {
                fields.span() => compile_error!("serde-devolve does not support data unions");
            }
            .into()
        }
    };

    let d = if is_tuple_struct {
        quote! {
            #[derive(::serde::Deserialize, ::serde::Serialize)]
            #serde_attrs
            #vis #devo_token #devo_name #ty_generics #devo_body #where_clause;
        }
    } else {
        quote! {
            #[derive(::serde::Deserialize, ::serde::Serialize)]
            #serde_attrs
            #vis #devo_token #devo_name #ty_generics #where_clause #devo_body
        }
    };
    quote! {
        #d

        impl #impl_generics ::serde_devo::Devolvable<#fallback_type> for #name #ty_generics #where_clause {
            type Devolved = #devo_name #ty_generics;

            fn devolve(self) -> Self::Devolved {
                #devo_impl
            }
        }

        impl #impl_generics ::serde_devo::Evolvable<#fallback_type> for #devo_name #ty_generics #where_clause {
            type Evolved = #name #ty_generics;

            fn try_evolve(self) -> Result<Self::Evolved, ::serde_devo::Error> {
                #evo_impl
            }
        }

        mod #warnings_mod {
            #(
                #warn
            )*
        }
    }
    .into()
}

fn render_variant(
    evo_name: &Ident,
    devo_name: &Ident,
    Variant {
        attrs,
        ident,
        fields,
        ..
    }: Variant,
    devo_attr: &Ident,
    fallback_type: &Type,
) -> (
    bool,
    TokenStream,
    Vec<TokenStream>,
    TokenStream,
    TokenStream,
) {
    let mut warn = vec![];
    let is_empty = fields.is_empty();
    let field_names = fields
        .iter()
        .filter_map(|f| f.ident.as_ref())
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(", ")
        .parse::<TokenStream>()
        .unwrap();
    let field_letters = fields
        .iter()
        .scan(b'a', |letter, _| {
            let l = *letter;
            *letter += 1;
            Some(l as char)
        })
        .map(|n| n.to_string())
        .collect::<Vec<_>>();
    let (is_devo, is_untagged, attrs) = render_attrs(attrs, devo_attr);
    let (is_named, tokens, e_impl, d_impl): (bool, TokenStream, TokenStream, TokenStream) =
        fields.into_iter().zip(&field_letters).enumerate().fold(
            (
                false,
                TokenStream::new(),
                TokenStream::new(),
                TokenStream::new(),
            ),
            |(b, mut st, mut evo, mut dvo), (i, (f, l))| {
                let is_named = f.ident.is_some();
                let (_is_devo, tokens, ev, dv) = if is_named {
                    render_field(f, devo_attr, true, fallback_type)
                } else {
                    render_tuple_field(f, devo_attr, i, Some(l), fallback_type)
                };
                st.append_all(tokens);
                evo.append_all(ev);
                dvo.append_all(dv);
                (is_named || b, st, evo, dvo)
            },
        );

    let field_letters = field_letters.join(", ").parse::<TokenStream>().unwrap();
    let tokens = if is_named {
        quote! {
            #attrs
            #ident {
                #tokens
            },
        }
    } else if is_empty {
        quote! {
            #attrs
            #ident,
        }
    } else {
        quote! {
            #attrs
            #ident (
                #tokens
            ),
        }
    };

    let member = format!("Self::{}", ident).parse::<TokenStream>().unwrap();
    let evo_member = format!("{}::{}", evo_name, ident)
        .parse::<TokenStream>()
        .unwrap();
    let devo_member = format!("{}::{}", devo_name, ident)
        .parse::<TokenStream>()
        .unwrap();
    let devo_impl = if is_empty {
        quote! {
            #member => Ok(#evo_member),
        }
    } else if is_named {
        quote! {
            #member { #field_names, .. } => Ok(#evo_member { #e_impl }),
        }
    } else {
        quote! {
            #member ( #field_letters ) => Ok(#evo_member ( #e_impl )),
        }
    };
    let evo_impl = if is_empty {
        quote! {
            #member => #devo_member,
        }
    } else if is_named {
        quote! {
            #member { #field_names } => #devo_member { #d_impl },
        }
    } else {
        quote! {
            #member ( #field_letters, .. ) => #devo_member ( #d_impl ),
        }
    };

    let span = ident.span();
    if is_named && is_devo {
        warn.push(
            syn::Error::new(
                span,
                "#[devo] does nothing on enum variants with named fields",
            )
            .into_compile_error(),
        );
    }

    if is_empty && is_devo {
        warn.push(
            syn::Error::new(span, "#[devo] does nothing on unit enum variants")
                .into_compile_error(),
        );
    }

    (is_untagged, tokens, warn, evo_impl, devo_impl)
}

fn render_tuple_field(
    Field { vis, attrs, ty, .. }: Field,
    devo_attr: &Ident,
    i: usize,
    l: Option<&str>,
    fallback_type: &Type,
) -> (bool, TokenStream, TokenStream, TokenStream) {
    let ty = &ty;
    let (is_devo, _, attrs) = render_attrs(attrs, devo_attr);
    let member = (if let Some(l) = l {
        l.to_string()
    } else {
        format!("self.{}", i)
    })
    .parse::<TokenStream>()
    .unwrap();
    if is_devo {
        if let Some(id) = match ty {
            Type::Path(p) => p.path.get_ident(),
            _ => None,
        } {
            let devolved_ident = format_ident!("{}", id);
            return (
                is_devo,
                quote! {
                    #attrs
                    #vis <#devolved_ident as ::serde_devo::Devolvable<#fallback_type>>::Devolved,
                },
                quote! {
                    <<#devolved_ident as ::serde_devo::Devolvable<#fallback_type>>::Devolved as ::serde_devo::Evolvable<#fallback_type>>::try_evolve(#member)?,
                },
                quote! {
                    <#devolved_ident as ::serde_devo::Devolvable<#fallback_type>>::devolve(#member),
                },
            );
        }
    }

    (
        is_devo,
        quote! {
            #attrs
            #vis #ty,
        },
        quote! {
            #member,
        },
        quote! {
            #member,
        },
    )
}

fn render_field(
    Field {
        vis,
        attrs,
        ident,
        ty,
        ..
    }: Field,
    devo_attr: &Ident,
    is_enum: bool,
    fallback_type: &Type,
) -> (bool, TokenStream, TokenStream, TokenStream) {
    let ty = &ty;
    let (is_devo, _, attrs) = render_attrs(attrs, devo_attr);
    let member = (if is_enum {
        format!("{}", ident.as_ref().unwrap())
    } else {
        format!("self.{}", ident.as_ref().unwrap())
    })
    .parse::<TokenStream>()
    .unwrap();
    if is_devo {
        if let Some(id) = match ty {
            Type::Path(p) => p.path.get_ident(),
            _ => None,
        } {
            let devolved_ident = format_ident!("{}", id);
            return (
                is_devo,
                quote! {
                    #attrs
                    #vis #ident: <#devolved_ident as ::serde_devo::Devolvable<#fallback_type>>::Devolved,
                },
                quote! {
                    #ident: <<#devolved_ident as ::serde_devo::Devolvable<#fallback_type>>::Devolved as ::serde_devo::Evolvable<#fallback_type>>::try_evolve(#member)?,
                },
                quote! {
                    #ident: <#devolved_ident as ::serde_devo::Devolvable<#fallback_type>>::devolve(#member),
                },
            );
        }
    }

    (
        is_devo,
        quote! {
            #attrs
            #vis #ident: #ty,
        },
        quote! {
            #ident: #member,
        },
        quote! {
            #ident: #member,
        },
    )
}

fn render_attrs(
    attrs: impl IntoIterator<Item = Attribute>,
    devo_attr: &Ident,
) -> (bool, bool, TokenStream) {
    let (is_devo, is_untagged, tokens) = attrs.into_iter().fold(
        (false, false, vec![]),
        |(b, mut t, mut v), attr| match &attr.meta {
            Meta::Path(name) if name.get_ident() == Some(devo_attr) => (true, t, v),
            Meta::List(list) if list.path.get_ident() == Some(&format_ident!("serde")) => {
                let _ = list.parse_nested_meta(|meta| {
                    if meta.path.is_ident("untagged") {
                        t = true
                    }
                    Ok(())
                });
                v.push(quote! {
                    #attr
                });
                (b, t, v)
            }
            _ => (b, t, v),
        },
    );

    (is_devo, is_untagged, tokens.into_iter().collect())
}

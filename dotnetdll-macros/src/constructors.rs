use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Bracket;
use syn::{bracketed, parenthesized, Ident, Token};

#[derive(Debug)]
pub enum External {
    Move(Ident),
    Clone(Ident),
}
impl Parse for External {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![#]) {
            input.parse::<Token![#]>()?;
            input.parse().map(External::Move)
        } else if lookahead.peek(Token![@]) {
            input.parse::<Token![@]>()?;
            input.parse().map(External::Clone)
        } else {
            Err(lookahead.error())
        }
    }
}
impl ToTokens for External {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            External::Move(i) => i.to_tokens(tokens),
            External::Clone(i) => tokens.append_all(quote! { #i.clone() }),
        }
    }
}

pub enum Type {
    Bare(Ident),
    External(External),
    Pointer(Box<Self>),
    Vector(Box<Self>),
}
impl Parse for Type {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        let mut result = if lookahead.peek(Ident::peek_any) {
            Type::Bare(input.parse()?)
        } else if lookahead.peek(Token![#]) || lookahead.peek(Token![@]) {
            Type::External(input.parse()?)
        } else {
            return Err(lookahead.error());
        };

        loop {
            let has_bracket = input.peek(Bracket);
            if has_bracket {
                let content;
                bracketed!(content in input);
                // TODO: allow shaped arrays
                if !content.is_empty() {
                    return Err(content.error("only empty arrays are currently allowed"));
                } else {
                    result = Type::Vector(Box::new(result));
                }
            }
            let has_asterisk = input.peek(Token![*]);
            if has_asterisk {
                input.parse::<Token![*]>()?;
                result = Type::Pointer(Box::new(result));
            }
            if !has_bracket && !has_asterisk {
                break;
            }
        }

        Ok(result)
    }
}
impl ToTokens for Type {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(ctype(self));
    }
}

macro_rules! check_idents {
    (match $i:ident { $($src:literal => $dest:ident,)* }) => {
        $(if $i == $src { quote! { BaseType::$dest.into() } } else )+ {
            panic!("unknown type {:?}", $i)
        }
    }
}

pub fn ctype(t: &Type) -> TokenStream {
    match t {
        Type::Bare(i) => check_idents!(match i {
            "bool" => Boolean,
            "char" => Char,
            "sbyte" => Int8,
            "byte" => UInt8,
            "short" => Int16,
            "ushort" => UInt16,
            "int" => Int32,
            "uint" => UInt32,
            "long" => Int64,
            "ulong" => UInt64,
            "float" => Float32,
            "double" => Float64,
            "nint" => IntPtr,
            "nuint" => UIntPtr,
            "object" => Object,
            "string" => String,
        }),
        Type::External(e) => quote! { #e },
        Type::Pointer(inner) => {
            if matches!(&**inner, Type::Bare(i) if i == "void") {
                quote! { BaseType::VOID_PTR.into() }
            } else {
                let t = ctype(inner);
                quote! { BaseType::pointer(#t).into() }
            }
        }
        Type::Vector(inner) => {
            let t = ctype(inner);
            quote! { BaseType::vector(#t).into() }
        }
    }
}

pub enum Parameter {
    Value(Type),
    Ref(Type),
    // if you need a typedref in your method signature, you can deal with constructing the signature yourself
}
impl Parse for Parameter {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![ref]) {
            input.parse::<Token![ref]>()?;
            input.parse().map(Parameter::Ref)
        } else {
            input.parse().map(Parameter::Value)
        }
    }
}
impl ToTokens for Parameter {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(match self {
            Parameter::Value(t) => quote! { ParameterType::Value(#t) },
            Parameter::Ref(t) => quote! { ParameterType::Ref(#t) },
        });
    }
}

mod kw {
    syn::custom_keyword!(void);
}
pub struct ReturnType(Option<Parameter>);
impl Parse for ReturnType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(ReturnType(if input.peek(kw::void) {
            input.parse::<kw::void>()?;
            None
        } else {
            Some(input.parse()?)
        }))
    }
}

pub struct ParameterList(Punctuated<Parameter, Token![,]>);
impl Parse for ParameterList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);
        Ok(ParameterList(Punctuated::parse_terminated(&content)?))
    }
}

pub struct Signature {
    is_static: Option<Token![static]>,
    return_type: ReturnType,
    parameters: ParameterList,
}
impl Parse for Signature {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            is_static: input.parse()?,
            return_type: input.parse()?,
            parameters: input.parse()?,
        })
    }
}

pub fn msig(sig: Signature) -> TokenStream {
    let constructor = if sig.is_static.is_none() {
        quote! { instance }
    } else {
        quote! { static_member }
    };

    let return_type = match sig.return_type.0 {
        None => quote! { ReturnType::VOID },
        Some(p) => quote! { ReturnType::new(#p) },
    };

    let params = sig.parameters.0.into_iter();

    quote! {
        MethodSignature::#constructor(#return_type, vec![#(Parameter::new(#params)),*])
    }
}

pub struct TypeName(Punctuated<Ident, Token![.]>);
impl Parse for TypeName {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Punctuated::parse_separated_nonempty(input).map(TypeName)
    }
}

pub fn type_name(TypeName(qualified): TypeName) -> TokenStream {
    let mut names: Vec<_> = qualified.into_iter().collect();

    let name = names.pop();
    let namespace = if names.is_empty() {
        quote! { None }
    } else {
        quote! { Some(stringify!(#(#names).*)) }
    };

    quote! { (#namespace, stringify!(#name)) }
}

pub struct TypeRef(TypeName, External);
impl Parse for TypeRef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![in]>()?;
        let assembly = input.parse()?;
        Ok(TypeRef(name, assembly))
    }
}

pub fn type_ref(TypeRef(typename, asm): TypeRef) -> TokenStream {
    let tn = type_name(typename);
    quote! {{
        let (ns, name) = #tn;
        ExternalTypeReference::new(ns.map(Into::into), name.into(), ResolutionScope::Assembly(#asm))
    }}
}

// just handle stuff like .ctor and .cctor for now
pub struct MethodName(Option<Token![.]>, Ident);
impl Parse for MethodName {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(MethodName(input.parse()?, input.parse()?))
    }
}
impl ToTokens for MethodName {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
        self.1.to_tokens(tokens);
    }
}

pub struct MethodRef {
    parent: External,
    name: MethodName,
    signature: Signature,
}
impl Parse for MethodRef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let is_static = input.parse()?;
        let return_type = input.parse()?;
        let parent = input.parse()?;
        input.parse::<Token![::]>()?;
        let name = input.parse()?;
        let parameters = input.parse()?;

        Ok(MethodRef {
            parent,
            name,
            signature: Signature {
                is_static,
                return_type,
                parameters,
            },
        })
    }
}

pub fn method_ref(meth: MethodRef) -> TokenStream {
    let sig = msig(meth.signature);
    let name = meth.name;
    let parent = meth.parent;

    quote! {
        ExternalMethodReference::new(
            MethodReferenceParent::Type(#parent),
            stringify!(#name).into(),
            #sig
        )
    }
}

pub struct FieldRef {
    field_type: Type,
    parent: External,
    name: Ident,
}
impl Parse for FieldRef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            field_type: input.parse()?,
            parent: input.parse()?,
            name: {
                input.parse::<Token![::]>()?;
                input.parse()?
            },
        })
    }
}

pub fn field_ref(
    FieldRef {
        field_type,
        parent,
        name,
    }: FieldRef,
) -> TokenStream {
    quote! {
        ExternalFieldReference::new(
            FieldReferenceParent::Type(#parent),
            #field_type,
            stringify!(#name).into()
        )
    }
}

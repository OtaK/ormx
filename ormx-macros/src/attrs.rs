use std::marker::PhantomData;

use syn::{
    Attribute, Ident, Path, Result, Token, Type,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

pub enum TableAttr {
    /// table = <string>
    Table(String),
    /// id = <ident>
    Id(Ident),
    /// insertable [= [<attribute>]* <ident>]?
    Insertable(Option<Insertable>),
    /// engine = <string>
    Engine(String),
    /// charset = <string>
    Charset(String),
    /// collation = <string>
    Collation(String),
    /// syncable
    Syncable(())
}

pub struct Insertable {
    pub attrs: Vec<Attribute>,
    pub ident: Ident,
}

pub enum TableFieldDefaultValue<'q, L> where L: sqlx_core::encode::Encode<'q, sqlx_core::any::Any> {
    LiteralValue(L),
    LiteralSQL(String),
    Deferred(fn() -> String),
    None,
    Unreachable(&'q PhantomData<L>),
}

impl<'q, L: sqlx_core::encode::Encode<'q, sqlx_core::any::Any>> sqlx_core::encode::Encode<'q, sqlx_core::any::Any> for TableFieldDefaultValue<'q, L> {
    fn encode_by_ref(&self, buf: &mut sqlx_core::any::AnyArgumentBuffer) -> sqlx_core::encode::IsNull {
        match self {
            Self::LiteralValue(l) => {
                l.encode_by_ref(buf)
            },
            Self::LiteralSQL(s) => {
                <String as sqlx_core::encode::Encode<'q, sqlx_core::any::Any>>::encode_by_ref(&s, buf)
            },
            Self::Deferred(f) => {
                let s = f();

                <String as sqlx_core::encode::Encode<'q, sqlx_core::any::Any>>::encode_by_ref(&s, buf)
            },
            Self::None => sqlx_core::encode::IsNull::Yes,
            Self::Unreachable(_) => unreachable!(),
        }
    }
}

pub enum TableFieldAttr {
    /// column = <string>
    Column(String),
    /// column_type = <string>
    ColumnType(String),
    /// custom_type
    CustomType(()),
    /// primary_key
    PrimaryKey(()),
    /// auto_increment
    AutoIncrement(()),
    /// allow_null = <bool>
    AllowNull(bool),
    /// unique [= <string>]
    Unique(Option<String>),
    /// default = <string>
    Default(String),
    /// get_one [= <ident>]? [(<type>)]?
    GetOne(Getter),
    /// get_optional [= <ident>]? [(<type>)]?
    GetOptional(Getter),
    /// get_many [= <ident>]? [(<type>)]?
    GetMany(Getter),
    /// set [= <ident>]?
    Set(Option<Ident>),
}

#[derive(Clone)]
pub struct Getter {
    pub func: Option<Ident>,
    pub arg_ty: Option<Type>,
}

pub enum PatchAttr {
    // table = <string>
    TableName(String),
    Table(Path),
    Id(String),
}

pub enum PatchFieldAttr {
    // column = <string>
    Column(String),
}

impl Parse for Getter {
    fn parse(input: ParseStream) -> Result<Self> {
        let func = if input.peek(syn::token::Eq) {
            input.parse::<syn::token::Eq>()?;
            Some(input.parse::<Ident>()?)
        } else {
            None
        };
        let arg_ty = if input.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);
            Some(content.parse::<Type>()?)
        } else {
            None
        };
        Ok(Getter { func, arg_ty })
    }
}

impl Parse for Insertable {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            attrs: input.call(Attribute::parse_outer)?,
            ident: input.parse()?,
        })
    }
}

pub fn parse_attrs<A: Parse>(attrs: &[Attribute]) -> Result<Vec<A>> {
    let attrs = attrs
        .iter()
        .filter(|a| a.path.is_ident("ormx"))
        .map(|a| a.parse_args_with(Punctuated::<A, Token![,]>::parse_terminated))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect();
    Ok(attrs)
}

/// implements `syn::parse::Parse` for the given type
macro_rules! impl_parse {
    // entry point
    ($i:ident {
        $( $s:literal => $v:ident( $($t:tt)* ) ),*
    }) => {
        impl syn::parse::Parse for $i {
            #[allow(clippy::redundant_closure_call)]
            fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                let ident = input.parse::<syn::Ident>()?;
                match &*ident.to_string() {
                    $( $s => (impl_parse!($($t)*))(input).map(Self::$v), )*
                    _ => Err(input.error("unknown attribute"))
                }
            }
        }
    };
    () => ( |_: ParseStream| Ok(()) );
    // parse either "= {value}" or return None
    ((= $t:tt)?) => {
        |i: ParseStream| if i.peek(syn::Token![=]) {
            i.parse::<syn::Token![=]>()?;
            #[allow(clippy::redundant_closure_call)]
            (impl_parse!($t))(i).map(Some)
        } else {
            Ok(None)
        }
    };
    // parse "= {value}"
    (= $x:tt) => ( |i: ParseStream| {
        i.parse::<syn::Token![=]>()?;
        #[allow(clippy::redundant_closure_call)]
        (impl_parse!($x))(i)
    } );
    (String) => ( |i: ParseStream| i.parse().map(|s: syn::LitStr| s.value()) );
    (bool) => ( |i: ParseStream| i.parse().map(|s: syn::LitBool| s.value) );
    ($t:ty) => ( |i: ParseStream| i.parse::<$t>() );
}

impl_parse!(TableAttr {
    "table" => Table(= String),
    "id" => Id(= Ident),
    "insertable" => Insertable((= Insertable)?),
    "engine" => Engine(= String),
    "charset" => Charset(= String),
    "collation" => Collation(= String),
    "syncable" => Syncable()
});

impl_parse!(TableFieldAttr {
    "column" => Column(= String),
    "column_type" => ColumnType(= String),
    "get_one" => GetOne(Getter),
    "get_optional" => GetOptional(Getter),
    "get_many" => GetMany(Getter),
    "set" => Set((= Ident)?),
    "custom_type" => CustomType(),
    "primary_key" => PrimaryKey(),
    "allow_null" => AllowNull(= bool),
    "auto_increment" => AutoIncrement(),
    "unique" => Unique((= String)?),
    "default" => Default(= String)
});

impl_parse!(PatchAttr {
    "table" => Table(= Path),
    "table_name" => TableName(= String),
    "id" => Id(= String)
});

impl_parse!(PatchFieldAttr {
    "column" => Column(= String)
});

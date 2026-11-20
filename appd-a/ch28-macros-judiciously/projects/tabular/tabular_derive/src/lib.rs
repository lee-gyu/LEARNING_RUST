//! `#[derive(Columns)]`를 처리하는 절차적 매크로 크레이트입니다.
//!
//! 구조체의 필드를 훑어 `tabular::Columns`의 열 이름(`NAMES`)과 칸(`cells`)을 함께 만듭니다.
//! 둘이 같은 필드 목록에서 나오므로 필드를 더하거나 지워도 어긋나지 않습니다.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields, LitStr, parse_macro_input};

/// 이름 붙은 필드를 가진 구조체에 `tabular::Columns`를 구현합니다. 칸은 필드의 `Display` 구현으로 만듭니다.
///
/// 필드에 붙일 수 있는 보조 속성(helper attribute):
/// - `#[column(rename = "이름")]`: 열 이름을 바꿉니다. 붙이지 않으면 필드 이름을 씁니다.
/// - `#[column(skip)]`: 표에서 뺍니다.
#[proc_macro_derive(Columns, attributes(column))]
pub fn derive_columns(input: TokenStream) -> TokenStream {
    // parse_macro_input!은 해석에 실패하면 컴파일 에러를 담은 TokenStream을 "이 함수에서" return합니다.
    // 숨은 return이 있는 매크로지만, syn 문서가 그 동작과 "TokenStream을 돌려주는 함수에서만 부를 것"을 밝혀 둡니다.
    let input = parse_macro_input!(input as DeriveInput);

    // 실패는 패닉 대신 syn::Error로 모았다가 compile_error!로 바꿉니다. 에러가 문제의 토큰을 가리킵니다.
    expand(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn expand(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let name = &input.ident;

    let Data::Struct(data) = &input.data else {
        return Err(syn::Error::new_spanned(
            name,
            "Columns는 구조체에만 derive할 수 있습니다",
        ));
    };
    let Fields::Named(fields) = &data.fields else {
        return Err(syn::Error::new_spanned(
            name,
            "Columns는 이름 붙은 필드가 있는 구조체에만 derive할 수 있습니다",
        ));
    };

    let mut names = Vec::new();
    let mut cells = Vec::new();
    for field in &fields.named {
        let ident = field
            .ident
            .as_ref()
            .expect("이름 붙은 필드에는 이름이 있습니다");
        let mut column = ident.to_string();
        let mut skip = false;

        for attr in field.attrs.iter().filter(|a| a.path().is_ident("column")) {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("skip") {
                    skip = true;
                    Ok(())
                } else if meta.path.is_ident("rename") {
                    column = meta.value()?.parse::<LitStr>()?.value();
                    Ok(())
                } else {
                    Err(meta.error("column 속성에는 `skip`이나 `rename = \"…\"`만 쓸 수 있습니다"))
                }
            })?;
        }

        if skip {
            continue;
        }
        names.push(column);
        // 칸을 만드는 코드에 필드 타입의 위치(span)를 붙입니다. 필드 타입이 Display를 구현하지 않으면
        // 에러가 #[derive(Columns)]가 아니라 그 필드의 타입을 가리킵니다.
        cells.push(quote_spanned! { field.ty.span() =>
            ::std::string::ToString::to_string(&self.#ident)
        });
    }

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // 만든 코드는 쓰는 쪽 크레이트에서 해석됩니다. 절차적 매크로에는 $crate가 없으므로 트레이트를
    // `::tabular::Columns`처럼 맨 앞의 `::`부터 적습니다. 쓰는 쪽이 트레이트를 use하지 않았어도 동작합니다.
    // `Vec`, `String`, `vec!`도 쓰는 쪽에서 가려질 수 있으므로 `::std::`부터 적습니다.
    Ok(quote! {
        impl #impl_generics ::tabular::Columns for #name #ty_generics #where_clause {
            const NAMES: &'static [&'static str] = &[#(#names),*];

            fn cells(&self) -> ::std::vec::Vec<::std::string::String> {
                ::std::vec![#(#cells),*]
            }
        }
    })
}

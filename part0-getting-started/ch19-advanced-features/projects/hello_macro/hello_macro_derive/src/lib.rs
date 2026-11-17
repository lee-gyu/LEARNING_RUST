//! `#[derive(HelloMacro)]`를 처리하는 절차적 매크로 크레이트입니다.
//!
//! 이름은 관례에 따라 "트레이트 크레이트 이름 + `_derive`"로 지었습니다.

use proc_macro::TokenStream;
use quote::quote;

// #[proc_macro_derive(HelloMacro)]: 누군가 #[derive(HelloMacro)]를 붙이면 컴파일러가 이 함수를 부릅니다.
// input은 derive를 붙인 항목(예: `struct Pancakes;`)의 토큰들이고,
// 돌려준 토큰들은 그 항목 바로 뒤에 덧붙여집니다.
#[proc_macro_derive(HelloMacro)]
pub fn hello_macro_derive(input: TokenStream) -> TokenStream {
    // 토큰을 구문 트리로 해석합니다. derive는 구조체·열거형·유니언에만 붙일 수 있어 실패할 일이 드물지만,
    // 실패하면 패닉으로 컴파일을 멈춥니다. 절차적 매크로는 Result가 아니라 TokenStream을 돌려줘야 하기 때문입니다.
    let ast = syn::parse(input).unwrap();

    impl_hello_macro(&ast)
}

fn impl_hello_macro(ast: &syn::DeriveInput) -> TokenStream {
    // ast.ident는 derive를 붙인 타입의 이름입니다(예: Pancakes).
    let name = &ast.ident;

    // quote! 안의 #name 자리에 위의 name이 들어갑니다.
    // stringify!(#name)은 컴파일 시점에 이름을 문자열 리터럴 "Pancakes"로 바꿉니다.
    let generated = quote! {
        impl HelloMacro for #name {
            fn hello_macro() {
                println!("안녕하세요, 매크로! 제 이름은 {}입니다!", stringify!(#name));
            }
        }
    };

    // quote!가 만든 것은 proc_macro2::TokenStream이라서 컴파일러가 받는 proc_macro::TokenStream으로 바꿉니다.
    generated.into()
}

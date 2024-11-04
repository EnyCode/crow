use proc_macro::TokenStream;
use quote::quote;
use syn::ItemFn;

#[proc_macro_attribute]
pub fn slash_command(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let function = syn::parse_macro_input!(item as syn::ItemFn);

    let name = &function.sig.ident;
    let content = &function.block;

    // Build the trait implementation

    let gen = quote! {
        fn #name() -> crate::client::interactions::SlashCommand {
            fn inner(ctx: InteractionContext<'_>, text: String, user: User, channel: Channel) -> BoxFuture<'_, ()> {
                async move {
                    #content
                }.boxed()
            }

            crate::client::interactions::SlashCommand {
                command: stringify!(#name).to_string(),
                execute: inner,
            }
        }
    };

    gen.into()
}

#[proc_macro_attribute]
pub fn message_action(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let function = syn::parse_macro_input!(item as syn::ItemFn);

    let name = &function.sig.ident;
    let content = &function.block;

    // Build the trait implementation

    let gen = quote! {
        fn #name() -> crate::client::interactions::MessageAction {
            fn inner(ctx: InteractionContext<'_>, user: User, name: String, display_name: String, channel: Channel) -> BoxFuture<'_, ()> {
                async move {
                    #content
                }.boxed()
            }

            crate::client::interactions::MessageAction {
                action: stringify!(#name).to_string(),
                execute: inner,
            }
        }
    };

    gen.into()
}

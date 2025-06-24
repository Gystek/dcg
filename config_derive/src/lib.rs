use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, parse_macro_input, DeriveInput};

#[proc_macro_derive(Merge)]
pub fn derive_merge(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let fields = if let Data::Struct(data) = input.data {
	data.fields
    } else {
	panic!("#[derive(Merge)] may only be used with structs");
    };

    let field_names = fields.iter().map(|f| &f.ident);

    let merges = field_names.clone().map(|f| {
	quote! {
	    #f: other.#f.or(self.#f)
	}
    });

    let expansion = quote ! {
	impl #name {
	    pub fn merge(self, other: Self) -> Self {
		Self {
		    #(#merges,)*
		}
	    }
	}
    };

    TokenStream::from(expansion)
}

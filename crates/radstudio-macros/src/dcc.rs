use quote::quote;
use syn::DeriveInput;

pub fn derive_options_data(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let fields = match &input.data {
        syn::Data::Struct(data) if let syn::Fields::Named(fields) = &data.fields => &fields.named,
        _ => panic!("Only supports standard structs"),
    };

    let mut blocks = Vec::new();

    for field in fields {
        let Some(attr) = field.attrs.iter().find(|a| a.path().is_ident("dcc")) else {
            continue;
        };

        let field_ident = field.ident.as_ref().unwrap();
        let field_name = field_ident.to_string();
        let mut name = String::new();
        let mut msbuild = String::new();

        attr.parse_nested_meta(|meta| {
            let lit: syn::LitStr = meta.value()?.parse()?;
            if meta.path.is_ident("name") {
                name = lit.value();
            } else if meta.path.is_ident("msbuild") {
                msbuild = lit.value();
            } else {
                return Err(meta.error("Unsupported attribute"));
            };
            Ok(())
        })?;

        let syn::Type::Path(ty) = &field.ty else {
            panic!("Unsupported field type");
        };
        let ty_name = ty.path.segments.last().unwrap().ident.to_string();

        let block = match ty_name.as_str() {
            "Option" => quote! {
                let value = self.#field_ident.clone().unwrap_or_default();
                if !value.is_empty() {
                    result.push(Arg {ident: #field_name, name: #name, msbuild: #msbuild, value});
                };
            },
            "bool" => quote! {
                if self.#field_ident {
                    result.push(Arg {ident: #field_name, name: #name, msbuild: #msbuild, value: String::new()});
                };
            },
            "Vec" => quote! {
                if !self.#field_ident.is_empty() {
                    result.push(Arg {ident: #field_name, name: #name, msbuild: #msbuild, value: self.#field_ident.join(";")});
                };
            },
            _ => panic!("Unsupported field type: {ty_name}"),
        };

        blocks.push(block);
    }

    let struct_name = input.ident;

    Ok(quote! {
        impl #struct_name {
            pub fn data(&self) -> Vec<Arg> {
                let mut result = Vec::new();
                #(#blocks)*
                result
            }
        }
    })
}

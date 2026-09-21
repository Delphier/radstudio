use quote::quote;
use syn::{ImplItem, ItemImpl, ReturnType};

pub fn derive_data(input: ItemImpl) -> syn::Result<proc_macro2::TokenStream> {
    let mut blocks = Vec::new();
    for item in &input.items {
        let ImplItem::Fn(func) = item else {
            continue;
        };

        if func.sig.inputs.len() != 1 {
            continue;
        };

        let Some(receiver) = func.sig.receiver() else {
            continue;
        };

        if !matches!(receiver.kind, syn::ReceiverKind::Reference(_, _, None)) {
            continue;
        }

        if let ReturnType::Type(_, ty) = &func.sig.output
            && let syn::Type::Path(path) = ty.as_ref()
            && let Some(s) = path.path.segments.last()
            && s.ident == "Result"
        {
            continue;
        };

        let func_ident = &func.sig.ident;
        let func_name = func_ident.to_string();

        blocks.push(quote! {result.insert(#func_name, serde_json::to_value(self.#func_ident())?);});
    }

    let self_type = &input.self_ty;

    Ok(quote! {
        #input

        impl #self_type {
            pub fn data(&self) -> serde_json::Result<indexmap::IndexMap<&'static str, serde_json::Value>> {
                let mut result = indexmap::IndexMap::new();
                #(#blocks)*
                Ok(result)
            }
        }
    })
}

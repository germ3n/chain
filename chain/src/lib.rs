use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn document(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    
    let args_str = args.to_string();
    let mut summary = String::new();
    let mut returns = String::new();
    let mut params = Vec::<(String, String)>::new();
    
    // Parse summary - more flexible pattern matching
    if let Some(start) = args_str.find("summary") {
        if let Some(eq_pos) = args_str[start..].find('=') {
            let after_eq = start + eq_pos + 1;
            let remaining = args_str[after_eq..].trim();
            if remaining.starts_with('"') {
                if let Some(end_quote) = remaining[1..].find('"') {
                    summary = remaining[1..end_quote + 1].to_string();
                }
            }
        }
    }
    
    // Parse returns - more flexible pattern matching
    if let Some(start) = args_str.find("returns") {
        if let Some(eq_pos) = args_str[start..].find('=') {
            let after_eq = start + eq_pos + 1;
            let remaining = args_str[after_eq..].trim();
            if remaining.starts_with('"') {
                if let Some(end_quote) = remaining[1..].find('"') {
                    returns = remaining[1..end_quote + 1].to_string();
                }
            }
        }
    }
    
    // Parse params - improved to handle whitespace properly
    if let Some(params_start) = args_str.find("params") {
        if let Some(brace_start) = args_str[params_start..].find('{') {
            let brace_start = params_start + brace_start + 1;
            if let Some(brace_end) = args_str[brace_start..].find('}') {
                let params_content = &args_str[brace_start..brace_start + brace_end];
                
                // More careful parsing to preserve spaces in quotes
                let mut current_pos = 0;
                while current_pos < params_content.len() {
                    // Find the parameter name (before colon)
                    if let Some(colon_pos) = params_content[current_pos..].find(':') {
                        let param_name = params_content[current_pos..current_pos + colon_pos].trim().to_string();
                        
                        // Find the quoted description after the colon
                        let after_colon = current_pos + colon_pos + 1;
                        let remaining = params_content[after_colon..].trim();
                        
                        if remaining.starts_with('"') {
                            if let Some(end_quote) = remaining[1..].find('"') {
                                let param_desc = remaining[1..end_quote + 1].to_string();
                                params.push((param_name, param_desc));
                                
                                // Move past this parameter
                                current_pos = after_colon + end_quote + 2;
                                if let Some(comma_pos) = params_content[current_pos..].find(',') {
                                    current_pos += comma_pos + 1;
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        }
    }
    
    // Build the documentation conditionally
    let mut doc_parts = vec![];
    
    // Always add summary if we have one
    if !summary.is_empty() {
        doc_parts.push(quote! { #[doc = #summary] });
    }
    
    // Add parameters section only if we have params
    if !params.is_empty() {
        doc_parts.push(quote! { #[doc = ""] });
        doc_parts.push(quote! { #[doc = "# Parameters"] });
        for (name, desc) in &params {
            let param_doc = format!("* `{}` - {}", name, desc);
            doc_parts.push(quote! { #[doc = #param_doc] });
        }
    }
    
    // Add returns section only if we have a return description
    if !returns.is_empty() {
        doc_parts.push(quote! { #[doc = ""] });
        doc_parts.push(quote! { #[doc = "# Returns"] });
        doc_parts.push(quote! { #[doc = #returns] });
    }
    
    let result = quote! {
        #(#doc_parts)*
        #input_fn
    };

    result.into()
}
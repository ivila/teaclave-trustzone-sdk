// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Error, Expr, Ident, LitStr, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

/// Declare a supplicant plugin with name, UUID, init and invoke functions.
///
/// This macro generates the FFI shims and the `PluginMethod` static.
///
/// # Example
///
/// ```no_run
/// use optee_teec::{declare_supp_plugin, PluginParameters};
///
/// fn my_init() -> optee_teec::Result<()> {
///     println!("plugin init");
///     Ok(())
/// }
///
/// fn my_invoke(params: &mut PluginParameters) -> optee_teec::Result<()> {
///     Ok(())
/// }
///
/// declare_supp_plugin!(
///     name: "my_plugin",
///     uuid: "ef620757-fa2b-4f19-a1c4-6e51cfe4c0f9",
///     init: my_init,
///     invoke: my_invoke,
/// );
/// ```
#[proc_macro]
pub fn declare_supp_plugin(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeclarePluginInput);

    let init_ident = &input.init;
    let invoke_ident = &input.invoke;

    // Parse UUID string at compile time
    let uuid_str = input.uuid.value();
    let parsed_uuid = match uuid::Uuid::parse_str(&uuid_str) {
        Ok(u) => u,
        Err(e) => {
            return syn::parse::Error::new(input.uuid.span(), format!("invalid UUID: {}", e))
                .to_compile_error()
                .into();
        }
    };
    let (time_low, time_mid, time_hi_and_version, clock_seq_and_node) = parsed_uuid.as_fields();
    let name_bytes = input.name.value();
    let name_bytes_with_null = format!("{}\0", name_bytes);
    let name_lit = syn::LitByteStr::new(name_bytes_with_null.as_bytes(), input.name.span());

    quote!(
        const _: () = {
            use core::ffi::c_char;
            use optee_teec::raw::size_t;

            unsafe extern "C" fn __plugin_init() -> optee_teec::raw::TEEC_Result {
                match #init_ident() {
                    Ok(()) => optee_teec::raw::TEEC_SUCCESS,
                    Err(err) => err.raw_code(),
                }
            }
            unsafe extern "C" fn __plugin_invoke_inner(
                cmd: u32,
                sub_cmd: u32,
                data: *mut c_char,
                in_len: size_t,
                out_len: *mut size_t,
            ) -> optee_teec::Result<()> {
                let mut parameter = unsafe {
                    optee_teec::PluginParameters::from_raw(cmd, sub_cmd, data, in_len, out_len)?
                };
                #invoke_ident(&mut parameter)
            }
            unsafe extern "C" fn __plugin_invoke(
                cmd: u32,
                sub_cmd: u32,
                data: *mut c_char,
                in_len: size_t,
                out_len: *mut size_t,
            ) -> optee_teec::raw::TEEC_Result {
                match __plugin_invoke_inner(cmd, sub_cmd, data, in_len, out_len) {
                    Ok(()) => optee_teec::raw::TEEC_SUCCESS,
                    Err(err) => err.raw_code(),
                }
            }
            static PLUGIN_NAME: &[u8] = #name_lit;
            #[unsafe(no_mangle)]
            pub static mut plugin_method: optee_teec::raw::PluginMethod = {
                let plugin_uuid = optee_teec::raw::TEEC_UUID {
                    timeLow: #time_low,
                    timeMid: #time_mid,
                    timeHiAndVersion: #time_hi_and_version,
                    clockSeqAndNode: [#(#clock_seq_and_node),* ],
                };
                optee_teec::raw::PluginMethod {
                    name: PLUGIN_NAME.as_ptr() as *const _,
                    uuid: plugin_uuid,
                    init: __plugin_init,
                    invoke: __plugin_invoke,
                }
            };
        };
    )
    .into()
}

struct DeclarePluginInput {
    name: LitStr,
    uuid: LitStr,
    init: Ident,
    invoke: Ident,
}

impl Parse for DeclarePluginInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args = Punctuated::<NamedArg, Token![,]>::parse_terminated(input)?;
        if args.len() != 4 {
            return Err(Error::new(
                proc_macro2::Span::call_site(),
                "declare_supp_plugin takes 4 fields: `name`, `uuid`, `init`, `invoke`.",
            ));
        }

        let mut name = None;
        let mut uuid = None;
        let mut init = None;
        let mut invoke = None;
        for arg in args {
            match arg.key.to_string().as_str() {
                "name" => name = Some(syn::parse2(arg.value.into_token_stream())?),
                "uuid" => uuid = Some(syn::parse2(arg.value.into_token_stream())?),
                "init" => init = Some(syn::parse2(arg.value.into_token_stream())?),
                "invoke" => invoke = Some(syn::parse2(arg.value.into_token_stream())?),
                other => {
                    return Err(Error::new(
                        arg.key.span(),
                        format!("unknown field `{}`", other),
                    ));
                }
            }
        }

        Ok(Self {
            name: name
                .ok_or_else(|| Error::new(proc_macro2::Span::call_site(), "missing `name`"))?,
            uuid: uuid
                .ok_or_else(|| Error::new(proc_macro2::Span::call_site(), "missing `uuid`"))?,
            init: init
                .ok_or_else(|| Error::new(proc_macro2::Span::call_site(), "missing `init`"))?,
            invoke: invoke
                .ok_or_else(|| Error::new(proc_macro2::Span::call_site(), "missing `invoke`"))?,
        })
    }
}

struct NamedArg {
    key: Ident,
    value: Expr,
}

impl Parse for NamedArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let value: Expr = input.parse()?;

        Ok(Self { key, value })
    }
}

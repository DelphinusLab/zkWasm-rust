use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident, Data, Field, Type};
use proc_macro2::TokenStream as TokenStream2;

struct Fd {
    name: Ident,
    ty: Type,
}

struct Context {
    name: Ident,
    fields: Vec<Fd>,
}

impl From<Field> for Fd {
    fn from(f: Field) -> Self {
        Self {
            name: f.ident.unwrap(),
            ty: f.ty,
        }
    }
}

impl From<DeriveInput> for Context {
    fn from(input: DeriveInput) -> Self {
        let name = input.ident;
        let fields = match input.data {
            Data::Struct(r) => r.fields,
            _ => {panic!("Unsupported data type")}
        };
        let fds = fields.into_iter().map(Fd::from).collect();
        Self { name, fields: fds }
    }
}

impl Context {
    pub fn witnessobj_render(&self) -> TokenStream2 {
        let name = self.name.clone();
        let fields_writer = self.witness_writer();
        let fields_reader = self.witness_reader();
        quote!(
            impl WitnessObjWriter for #name {
                fn to_witness(&self, _ori_base: *const u8, _wit_base: *const u8) {
                    #(#fields_writer)*
                }
            }

            impl WitnessObjReader for #name {
                fn from_witness(obj: *mut Self) {
                    let obj_ptr = obj as *mut usize;
                    #(#fields_reader)*
                }
            }
        )
    }

    fn witness_reader(&self) -> Vec<TokenStream2> {
        let mut ret = vec![];
        for i in 0..self.fields.len() {
            let ty = self.fields[i].ty.clone();
            let name = self.fields[i].name.clone();
            ret.push(quote!(self.#name.from_witness(obj_ptr as *mut #ty);));
            ret.push(quote!(obj_ptr.add(sizeof(#ty));));
        }
        ret
    }

    fn witness_writer(&self) -> Vec<TokenStream2> {
        let mut ret = vec![];
        for i in 0..self.fields.len() {
            let name = self.fields[i].name.clone();
            ret.push(quote!(self.#name.to_witness(_ori_base, _wit_base);));
        }
        ret
    }
}

#[proc_macro_derive(WitnessObj)]
pub fn derive_witnessobj(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    Context::from(input).witnessobj_render().into()
}
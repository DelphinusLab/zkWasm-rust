use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Field, Ident, Variant};

struct Fd {
    name: Ident,
}

struct StructContext {
    name: Ident,
    fields: Vec<Fd>,
}

struct Ed {
    name: Ident,
    tname: Ident,
}

struct EnumContext {
    name: Ident,
    variants: Vec<Ed>,
}

enum Context {
    S(StructContext),
    E(EnumContext),
}


impl From<Field> for Fd {
    fn from(f: Field) -> Self {
        Self {
            name: f.ident.unwrap()
        }
    }
}

impl From<Variant> for Ed {
    fn from(f: Variant) -> Self {
        println!("fields {:?}", f.fields.iter().collect::<Vec<_>>()[0].clone().ty);
        Self {
            name: f.ident,
            tname: f.fields.iter().collect::<Vec<_>>()[0].clone().ident.expect("option none")
        }
    }
}

impl From<DeriveInput> for Context {
    fn from(input: DeriveInput) -> Self {
        let name = input.ident;
        match input.data {
            Data::Struct(r) => {
                let fds = r.fields.into_iter().map(Fd::from).collect();
                Self::S (StructContext { name, fields: fds })
            }
            Data::Enum(r) => {
                let variants = r.variants.into_iter().map(Ed::from).collect();
                Self::E (EnumContext { name, variants})
            }
            _ => {
                panic!("Unsupported data type")
            }
        }
    }
}

impl StructContext {
    pub fn witness_obj_render(&self) -> TokenStream2 {
        let name = self.name.clone();
        let fields_writer = self.witness_writer();
        let fields_reader = self.witness_reader();
        quote!(
            impl WitnessObjWriter for #name {
                fn to_witness(&self, ori_base: *const u8) {
                    #(#fields_writer)*
                }
            }

            impl WitnessObjReader for #name {
                fn from_witness(&mut self, fetcher: &mut impl FnMut() -> u64,  base: *const u8) {
                    unsafe {
                        #(#fields_reader)*
                    }
                }
            }
        )
    }

    fn witness_reader(&self) -> Vec<TokenStream2> {
        let mut ret = vec![];
        for i in 0..self.fields.len() {
            let name = self.fields[i].name.clone();
            ret.push(quote!(self.#name.from_witness(fetcher, base);));
        }
        ret
    }

    fn witness_writer(&self) -> Vec<TokenStream2> {
        let mut ret = vec![];
        for i in 0..self.fields.len() {
            let name = self.fields[i].name.clone();
            ret.push(quote!(self.#name.to_witness(ori_base);));
        }
        ret
    }
}


impl EnumContext {
    pub fn witness_obj_render(&self) -> TokenStream2 {
        let name = self.name.clone();
        let fields_writer = self.witness_writer();
        let fields_reader = self.witness_reader();
        quote!(
            impl WitnessObjWriter for #name {
                fn to_witness(&self, ori_base: *const u8) {
                    match self {
                        #(#fields_writer)*
                    }
                }
            }

            impl WitnessObjReader for #name {
                fn from_witness(&mut self, fetcher: &mut impl FnMut() -> u64,  base: *const u8) {
                    let enum_index = fetcher();
                    let ptr = unsafe { self as *const u64 };
                    *ptr = enum_index;
                    let obj_ptr = ptr.add(size_of(u64));
                    unsafe {
                        match enum_index {
                            #(#fields_reader)*
                        }
                    }
                }
            }
        )
    }

    fn witness_reader(&self) -> Vec<TokenStream2> {
        let mut ret = vec![];
        for i in 0..self.variants.len() {
            let name = self.variants[i].name.clone();
            let tname = self.variants[i].tname.clone();
            ret.push(quote!(
                #i => {
                    (obj_ptr as *mut #tname).from_witness(fetcher, base);
                }
            ));
        }
        ret
    }

    fn witness_writer(&self) -> Vec<TokenStream2> {
        let mut ret = vec![];
        for i in 0..self.variants.len() {
            let name = self.variants[i].name.clone();
            ret.push(quote!(
                #name(obj) => {
                    wasm_witness_insert(i)
                    obj.to_witness(ori_base);
                }
            ));
        }
        ret
    }
}


#[proc_macro_derive(WitnessObj)]
pub fn derive_witness_obj(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let c = Context::from(input);
    match c {
        Context::S(s) => s.witness_obj_render().into(),
        Context::E(e) => e.witness_obj_render().into()
    }
}

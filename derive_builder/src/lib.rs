use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Field, Ident, Variant, Type};

struct Fd {
    name: Ident,
}

struct StructContext {
    name: Ident,
    fields: Vec<Fd>,
}

struct Ed {
    name: Ident,
    ty: Type,
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
        let fields = f.fields.iter().collect::<Vec<_>>().clone();
        let t = fields[0].clone().ty;
        Self {
            name: f.ident,
            ty: t,
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
                fn to_witness(&self, witness_writer: &mut impl FnMut(u64), ori_base: *const u8) {
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
            ret.push(quote!(self.#name.to_witness(witness_writer, ori_base);));
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
                fn to_witness(&self, witness_writer: &mut impl FnMut(u64), ori_base: *const u8) {
                    unsafe {
                        let obj = self as *const Self;
                        let ptr = obj as *const u64;
                        let v = *ptr;
                        crate::dbg!("to witness of {:?}\n", self);
                        crate::dbg!("discriment is {}\n", v);
                    }
                    match self {
                        #(#fields_writer)*
                    }
                }
            }

            impl WitnessObjReader for #name {
                fn from_witness(&mut self, fetcher: &mut impl FnMut() -> u64,  base: *const u8) {
                    let obj = self as *mut Self;
                    let enum_index = fetcher();
                    crate::dbg!("enum index is {}\n", enum_index);
                    unsafe {
                        let ptr = obj as *mut u64;
                        *ptr = enum_index;
                        let obj_ptr = unsafe { ptr.add(1) };
                        match enum_index {
                            #(#fields_reader)*
                            _ => unreachable!()
                        }
                    }
                }
            }
        )
    }

    fn witness_reader(&self) -> Vec<TokenStream2> {
        let mut ret = vec![];
        for i in 0..self.variants.len() {
            let index = i as u64;
            let ty = self.variants[i].ty.clone();
            ret.push(quote!(
                #index => {
                    (*(obj_ptr as *mut #ty)).from_witness(fetcher, base);
                }
            ));
        }
        ret
    }

    fn witness_writer(&self) -> Vec<TokenStream2> {
        let mut ret = vec![];
        for i in 0..self.variants.len() {
            let index = i as u64;
            let name = self.variants[i].name.clone();
            ret.push(quote!(
                Self::#name(obj) => {
                    unsafe { witness_writer(#index) };
                    obj.to_witness(witness_writer, ori_base);
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

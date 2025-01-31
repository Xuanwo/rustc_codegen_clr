use crate::{
    access_modifier::AccessModifer,
    cil::FieldDescriptor,
    method::Method,
    r#type::{DotnetTypeRef, Type},
    IString,
};
use rustc_span::def_id::DefId;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub struct TypeDef {
    access: AccessModifer,
    name: IString,
    inner_types: Vec<Self>,
    fields: Vec<(IString, Type)>,
    functions: Vec<Method>,
    explicit_offsets: Option<Vec<u32>>,
    gargc: u32,
    extends: Option<DotnetTypeRef>,
}
impl TypeDef {
    #[must_use]
    pub fn ptr_components(name: &str, metadata: Type) -> Self {
        let mut ptr_components = crate::r#type::TypeDef::nameonly(name);
        ptr_components.add_field("data_address".into(), Type::Ptr(Type::Void.into()));
        ptr_components.add_field("metadata".into(), metadata);
        ptr_components
    }
    pub fn morphic_fields<'a>(
        &'a self,
        generics: &'a [Type],
    ) -> impl Iterator<Item = Option<(&'a str, Type)>> + 'a {
        self.fields()
            .iter()
            .map(|(name, tpe)| Some((name.as_ref(), tpe.map_generic(generics)?)))
    }
    pub fn set_generic_count(&mut self, generic_count: u32) {
        self.gargc = generic_count;
    }

    #[must_use]
    pub fn gargc(&self) -> u32 {
        self.gargc
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub fn access_modifier(&self) -> AccessModifer {
        self.access
    }
    #[must_use]
    pub fn extends(&self) -> Option<&DotnetTypeRef> {
        self.extends.as_ref()
    }
    #[must_use]
    pub fn fields(&self) -> &[(IString, Type)] {
        &self.fields
    }
    pub fn add_field(&mut self, name: IString, tpe: Type) {
        self.fields.push((name, tpe));
    }
    #[must_use]
    pub fn inner_types(&self) -> &[Self] {
        &self.inner_types
    }
    #[must_use]
    pub fn explicit_offsets(&self) -> Option<&Vec<u32>> {
        self.explicit_offsets.as_ref()
    }
    pub fn add_method(&mut self, method: Method) {
        self.functions.push(method);
    }
    pub fn methods(&self) -> impl Iterator<Item = &Method> {
        self.functions.iter()
    }
    #[must_use]
    pub fn nameonly(name: &str) -> Self {
        Self {
            access: AccessModifer::Public,
            name: name.into(),
            inner_types: vec![],
            fields: vec![],
            functions: vec![],
            gargc: 0,
            extends: None,
            explicit_offsets: None,
        }
    }
    #[must_use]
    pub fn new(
        access: AccessModifer,
        name: IString,
        inner_types: Vec<Self>,
        fields: Vec<(IString, Type)>,
        functions: Vec<Method>,
        explicit_offsets: Option<Vec<u32>>,
        gargc: u32,
        extends: Option<DotnetTypeRef>,
    ) -> Self {
        Self {
            access,
            name,
            inner_types,
            fields,
            functions,
            explicit_offsets,
            gargc,
            extends,
        }
    }
}
impl From<TypeDef> for Type {
    fn from(val: TypeDef) -> Type {
        Type::DotnetType(DotnetTypeRef::new(None, val.name()).into())
    }
}
impl From<&TypeDef> for Type {
    fn from(val: &TypeDef) -> Type {
        Type::DotnetType(DotnetTypeRef::new(None, val.name()).into())
    }
}
impl From<TypeDef> for DotnetTypeRef {
    fn from(val: TypeDef) -> DotnetTypeRef {
        DotnetTypeRef::new(None, val.name())
    }
}
impl From<&TypeDef> for DotnetTypeRef {
    fn from(val: &TypeDef) -> DotnetTypeRef {
        DotnetTypeRef::new(None, val.name())
    }
}
#[must_use]
pub fn escape_field_name(name: &str) -> IString {
    match name.chars().next() {
        None => "fld".into(),
        Some(first) => {
            if !(first.is_alphabetic() || first == '_')
        || name == "value"
        || name == "flags"
        || name == "alignment"
        || name == "init"
        || name == "string"
        || name == "nint"
        || name == "nuint"
        || name == "out"
        || name == "rem"
        || name == "add"
        || name == "div"
        || name == "error"
        || name == "opt"
        || name == "private"
        || name == "public"
        || name == "object"
        || name == "class"
        //FIXME: this is a sign of a bug. ALL fields not starting with a letter should have been caught by the statement above.
        || name == "0"
            {
                format!("m_{name}").into()
            } else {
                if name.contains('0') {
                    eprintln!(
                        "field name:\'{name:?}\'. Name length:{} first char:\'{:?}\'",
                        name.len(),
                        first
                    );
                }
                name.into()
            }
        }
    }
}
pub fn closure_name(def_id: DefId, fields: &[Type], sig: &crate::function_sig::FnSig) -> String {
    let mangled_fields: String = fields.iter().map(|f| crate::r#type::mangle(f)).collect();
    format!(
        "Closure{field_count}{mangled_fields}",
        field_count = fields.len()
    )
}
pub fn closure_typedef(def_id: DefId, fields: &[Type], sig: crate::function_sig::FnSig) -> TypeDef {
    let name = closure_name(def_id, fields, &sig);
    let fields = fields
        .iter()
        .enumerate()
        .map(|(idx, ty)| (format!("f{idx}").into(), ty.clone()))
        .collect();
    TypeDef::new(
        AccessModifer::Public,
        name.into(),
        vec![],
        fields,
        vec![],
        None,
        0,
        None,
    )
}
pub fn arr_name(element_count: usize, element: &Type) -> IString {
    let element_name = super::mangle(element);
    format!("Arr{element_count}_{element_name}",).into()
}
pub fn tuple_name(elements: &[Type]) -> IString {
    let generics: String = elements.iter().map(|ele| super::mangle(ele)).collect();
    format!(
        "Tuple{generic_count}{generics}",
        generic_count = generics.len()
    )
    .into()
}

#[must_use]
pub fn tuple_typedef(elements: &[Type]) -> TypeDef {
    let name = tuple_name(elements);
    let fields: Vec<_> = elements
        .iter()
        .enumerate()
        .map(|(idx, ele)| (format!("Item{}", idx + 1).into(), ele.clone()))
        .collect();
    TypeDef::new(
        AccessModifer::Public,
        name,
        vec![],
        fields,
        vec![],
        None,
        0,
        None,
    )
}
#[must_use]
pub fn get_array_type(element_count: usize, element: Type) -> TypeDef {
    use crate::cil::CILOp;
    let name = arr_name(element_count, &element);
    let mut fields = Vec::with_capacity(element_count);
    for field in 0..element_count {
        fields.push((format!("f_{field}").into(), element.clone()));
    }
    let mut def = TypeDef {
        access: AccessModifer::Public,
        name: name.into(),
        inner_types: vec![],
        fields,
        functions: vec![],
        explicit_offsets: None,
        gargc: 0,
        extends: None,
    };
    // set_Item(usize offset, G0 value)
    let mut set_usize = Method::new(
        AccessModifer::Public,
        false,
        crate::function_sig::FnSig::new(
            &[(&def).into(), Type::USize, element.clone()],
            &Type::Void,
        ),
        "set_Item",
        vec![],
    );
    let ops = vec![
        CILOp::LDArg(0),
        CILOp::LDFieldAdress(FieldDescriptor::boxed(
            (&def).into(),
            element.clone(),
            "f_0".to_string().into(),
        )),
        CILOp::LDArg(1),
        CILOp::Add,
        CILOp::LDArg(2),
        CILOp::STObj(element.clone().into()),
        CILOp::Ret,
    ];
    set_usize.set_ops(ops);
    def.add_method(set_usize);
    // get_Address(usize offset)
    let mut get_adress_usize = Method::new(
        AccessModifer::Public,
        false,
        crate::function_sig::FnSig::new(
            &[(&def).into(), Type::USize],
            &Type::Ptr(element.clone().into()),
        ),
        "get_Address",
        vec![],
    );
    let ops = vec![
        CILOp::LDArg(0),
        CILOp::LDFieldAdress(FieldDescriptor::boxed(
            (&def).into(),
            element.clone(),
            "f_0".to_string().into(),
        )),
        CILOp::LDArg(1),
        CILOp::Add,
        CILOp::Ret,
    ];
    get_adress_usize.set_ops(ops);
    def.add_method(get_adress_usize);
    // get_Item
    let mut get_item_usize = Method::new(
        AccessModifer::Public,
        false,
        crate::function_sig::FnSig::new(&[(&def).into(), Type::USize], &element.clone()),
        "get_Item",
        vec![],
    );
    let ops = vec![
        CILOp::LDArg(0),
        CILOp::LDFieldAdress(FieldDescriptor::boxed(
            (&def).into(),
            element.clone(),
            "f_0".to_string().into(),
        )),
        CILOp::LDArg(1),
        CILOp::Add,
        CILOp::LdObj(element.clone().into()),
        CILOp::Ret,
    ];
    get_item_usize.set_ops(ops);
    def.add_method(get_item_usize);
    def
}

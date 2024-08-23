use quote::quote;

use super::{Entity, EntityType};

pub fn file_parser(file: syn::File) -> Vec<Entity> {
    let mut entities: Vec<Entity> = vec![];
    for item in file.items {
        match item {
            syn::Item::Enum(item) => {
                entities.push(enum_parser(item));
            }
            syn::Item::Struct(item) => {
                entities.push(struct_parser(item));
            }
            _ => (),
        }
    }
    entities
}

fn enum_parser(item: syn::ItemEnum) -> Entity {
    let name = item.ident.to_string();
    let variants = item.variants.into_iter().map(variant_parser).collect();
    Entity::new(EntityType::Enum, &name, variants)
}

fn variant_parser(variant: syn::Variant) -> Entity {
    let name = variant.ident.to_string();
    if let syn::Fields::Unnamed(fields) = variant.fields {
        let mut fields = fields
            .unnamed
            .into_iter()
            .map(|field| type_parser(field.ty))
            .fold(String::new(), |acc, x| acc + &x + ", ");
        fields.pop();
        fields.pop();
        return Entity::new(EntityType::Variant(fields), &name, Vec::new());
    }
    Entity::new(EntityType::Variant("".to_string()), &name, Vec::new())
}

fn struct_parser(item: syn::ItemStruct) -> Entity {
    let name = item.ident.to_string();
    let fields = fields_parser(item.fields);
    Entity {
        entity_type: EntityType::Struct,
        name,
        fields,
    }
}

fn fields_parser(item: syn::Fields) -> Vec<Entity> {
    match item {
        syn::Fields::Named(syn::FieldsNamed { named: fields, .. }) => {
            fields.into_iter().map(field_parser).collect()
        }
        _ => vec![],
    }
}

fn field_parser(field: syn::Field) -> Entity {
    let name = field
        .ident
        .map(|ident| ident.to_string())
        .unwrap_or_else(|| "".to_string());

    let ty = type_parser(field.ty);
    if has_dependencies(&ty) {
        let fields = make_dependencies(&ty);
        return Entity::new(EntityType::Field(ty), &name, vec![fields]);
    }
    Entity::new(EntityType::Field(ty), &name, Vec::new())
}

fn type_parser(type_: syn::Type) -> String {
    let type_name = quote!(#type_);
    type_name
        .to_string()
        .chars()
        .filter(|c| *c != ' ')
        .collect()
}

fn has_dependencies(type_name: &str) -> bool {
    let cnt = type_name
        .chars()
        .filter(|x| (*x == ',') || (*x == '<') || (*x == '>'))
        .count();
    if cnt != 0 {
        return true;
    }
    false
}

fn make_dependencies(type_name: &str) -> Entity {
    let dependencies: Vec<&str> = type_name
        .split(|x| (x == ',') || (x == '<') || (x == '>'))
        .collect();
    let dependencies = dependencies
        .into_iter()
        .map(|x| x.to_string())
        .filter(|x| x != "")
        .map(|x| x.replace(" ", ""))
        .collect::<Vec<String>>();
    let dependencies = dependencies
        .into_iter()
        .map(|x| Entity::new(EntityType::Field("".to_string()), &x, Vec::new()))
        .collect::<Vec<Entity>>();
    Entity::new(EntityType::Struct, type_name, dependencies)
}

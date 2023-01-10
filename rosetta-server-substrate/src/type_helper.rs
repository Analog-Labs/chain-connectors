use crate::utils::Error;
use serde_json::Value;
use subxt::dynamic::Value as SubxtValue;
use subxt::ext::sp_runtime::scale_info::{
    form::PortableForm, TypeDef, TypeDefArray, TypeDefBitSequence, TypeDefCompact,
    TypeDefComposite, TypeDefPrimitive, TypeDefSequence, TypeDefTuple, TypeDefVariant,
};
use subxt::ext::sp_runtime::scale_info::{Field, PortableRegistry};

pub fn get_params(
    json_params: Vec<Value>,
    fields: Vec<Field<PortableForm>>,
    types: &PortableRegistry,
) -> Result<Vec<SubxtValue>, Error> {
    let mut vec_of_value = vec![];
    for (param, field) in json_params.iter().zip(fields) {
        let ty_id = field.ty().id();
        let type_from_pallet = get_type_def(ty_id, types)?;
        if let Ok(converted_type) = type_distributor(param.clone(), type_from_pallet, types) {
            vec_of_value.push(converted_type);
        };
    }
    Ok(vec_of_value)
}

pub fn get_type_def(id: u32, types: &PortableRegistry) -> Result<&TypeDef<PortableForm>, Error> {
    let ty = types.resolve(id).ok_or(Error::InvalidMetadata)?;
    Ok(ty.type_def())
}

pub fn type_distributor(
    json_value: Value,
    type_from_pallet: &TypeDef<PortableForm>,
    types: &PortableRegistry,
) -> Result<SubxtValue, Error> {
    match type_from_pallet {
        TypeDef::Variant(inner_val) => make_variant(json_value, inner_val, types),
        TypeDef::Composite(inner_val) => make_composite(json_value, inner_val, types),
        TypeDef::Sequence(inner_val) => make_sequence(json_value, inner_val),
        TypeDef::Array(inner_val) => make_array(json_value, inner_val),
        TypeDef::Tuple(inner_val) => make_tuple(json_value, inner_val),
        TypeDef::Primitive(inner_val) => make_primitive(json_value, inner_val),
        TypeDef::Compact(inner_val) => make_compact(json_value, inner_val),
        TypeDef::BitSequence(inner_val) => make_bit_sequence(json_value, inner_val),
    }
}

fn make_variant(
    json_value: Value,
    type_from_pallet: &TypeDefVariant<PortableForm>,
    types: &PortableRegistry,
) -> Result<SubxtValue, Error> {
    let variants = type_from_pallet.variants();
    let mut vec_of_named_data: Vec<(String, SubxtValue)> = vec![];
    let mut vec_of_unnamed_data: Vec<SubxtValue> = vec![];

    let json_key = json_value.as_array().ok_or(Error::InvalidParams)?[0]
        .as_str()
        .ok_or(Error::InvalidParams)?;
    let json_value = json_value.as_array().ok_or(Error::InvalidParams)?[1].clone();

    let fields = variants
        .iter()
        .find(|p| p.name == json_key)
        .ok_or(Error::InvalidVariantID)?
        .fields
        .clone();
    let is_named = fields.iter().any(|f| f.name().is_some());

    for field in fields {
        let ty_id = field.ty().id();
        let type_def = get_type_def(ty_id, types)?;
        let obtained_type = type_distributor(json_value.clone(), type_def, types);
        if let Ok(obtained_type) = obtained_type {
            if is_named {
                vec_of_named_data.push((
                    field.name().ok_or(Error::InvalidMetadata)?.to_string(),
                    obtained_type,
                ));
            } else {
                vec_of_unnamed_data.push(obtained_type);
            }
        }
    }

    if is_named {
        Ok(SubxtValue::named_variant(json_key, vec_of_named_data))
    } else {
        Ok(SubxtValue::unnamed_variant(json_key, vec_of_unnamed_data))
    }
}

fn make_composite(
    json_value: Value,
    type_from_pallet: &TypeDefComposite<PortableForm>,
    types: &PortableRegistry,
) -> Result<SubxtValue, Error> {
    let fields = type_from_pallet.fields();
    let mut vec_of_named_data: Vec<(String, SubxtValue)> = vec![];
    let mut vec_of_unnamed_data: Vec<SubxtValue> = vec![];

    let is_named = fields.iter().any(|f| f.name().is_some());

    for field in fields {
        let ty_id = field.ty().id();
        let type_def = get_type_def(ty_id, types)?;
        let obtained_type = type_distributor(json_value.clone(), type_def, types);
        if let Ok(obtained_type) = obtained_type {
            if is_named {
                vec_of_named_data.push((
                    field.name().ok_or(Error::InvalidMetadata)?.to_string(),
                    obtained_type,
                ));
            } else {
                vec_of_unnamed_data.push(obtained_type);
            }
        }
    }

    if is_named {
        Ok(SubxtValue::named_composite(vec_of_named_data))
    } else {
        Ok(SubxtValue::unnamed_composite(vec_of_unnamed_data))
    }
}

fn make_sequence(
    _json_value: Value,
    _type_from_pallet: &TypeDefSequence<PortableForm>,
) -> Result<SubxtValue, Error> {
    Err(Error::MakingCallParams)
}

fn make_array(
    json_value: Value,
    _type_from_pallet: &TypeDefArray<PortableForm>,
) -> Result<SubxtValue, Error> {
    if let Value::Array(val) = json_value {
        let mut vec_value = vec![];
        for value in val {
            let str_number = value.to_string();
            let parsed_number = str_number.parse::<u8>().map_err(|_| Error::InvalidParams)?;
            vec_value.push(parsed_number);
        }
        let referenced_vec = &vec_value;
        let bytes_data: &[u8] = referenced_vec;
        return Ok(SubxtValue::from_bytes(bytes_data));
    }
    Err(Error::MakingCallParams)
}

fn make_tuple(
    _json_value: Value,
    _type_from_pallet: &TypeDefTuple<PortableForm>,
) -> Result<SubxtValue, Error> {
    Err(Error::MakingCallParams)
}

fn make_primitive(
    json_value: Value,
    _type_from_pallet: &TypeDefPrimitive,
) -> Result<SubxtValue, Error> {
    match json_value {
        Value::Bool(val) => Ok(SubxtValue::bool(val)),
        Value::Number(val) => {
            let number_string = val.to_string();
            let number_i128 = number_string
                .parse::<u128>()
                .map_err(|_| Error::InvalidParams)?;
            Ok(SubxtValue::u128(number_i128))
        }
        Value::String(val) => Ok(SubxtValue::string(val)),
        _ => Err(Error::MakingCallParams),
    }
}

fn make_compact(
    json_value: Value,
    _type_from_pallet: &TypeDefCompact<PortableForm>,
) -> Result<SubxtValue, Error> {
    match json_value {
        Value::Number(val) => {
            let number_string = val.to_string();
            let number_i128 = number_string
                .parse::<u128>()
                .map_err(|_| Error::InvalidParams)?;
            Ok(SubxtValue::u128(number_i128))
        }
        _ => Err(Error::MakingCallParams),
    }
}

fn make_bit_sequence(
    _json_value: Value,
    _type_from_pallet: &TypeDefBitSequence<PortableForm>,
) -> Result<SubxtValue, Error> {
    Err(Error::MakingCallParams)
}

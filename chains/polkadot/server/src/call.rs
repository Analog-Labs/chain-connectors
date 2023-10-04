use anyhow::{Context, Result};
use scale_info::{
    form::PortableForm, PortableRegistry, TypeDef, TypeDefArray, TypeDefBitSequence,
    TypeDefCompact, TypeDefComposite, TypeDefPrimitive, TypeDefSequence, TypeDefTuple,
    TypeDefVariant,
};
use serde_json::{Map, Value, Value as SerdeValue};
use subxt::{
    dynamic::Value as SubxtValue,
    ext::scale_value::{self, scale::TypeId, BitSequence, ValueDef},
    metadata::types::StorageEntryType,
    OnlineClient, PolkadotConfig as GenericConfig,
};

pub fn dynamic_constant_req(
    subxt: &OnlineClient<GenericConfig>,
    pallet_name: &str,
    constant_name: &str,
) -> Result<Value> {
    let constant_address = subxt::dynamic::constant(pallet_name, constant_name);
    let data = subxt.constants().at(&constant_address)?.to_value()?;

    let serde_val = scale_to_serde_json(data.value)?;
    Ok(serde_val)
}

pub async fn dynamic_storage_req(
    subxt: &OnlineClient<GenericConfig>,
    pallet_name: &str,
    storage_name: &str,
    params: Value,
) -> Result<Value> {
    let metadata = subxt.metadata();
    let types = metadata.types();
    let pallet = metadata
        .pallet_by_name(pallet_name)
        .ok_or_else(|| anyhow::anyhow!("pallet not found"))?;
    let storage_metadata = pallet
        .storage()
        .and_then(|s| s.entry_by_name(storage_name))
        .ok_or_else(|| anyhow::anyhow!("storage not found"))?;

    let storage_type = storage_metadata.entry_type().clone();
    let type_id = match storage_type {
        StorageEntryType::Map { key_ty, .. } => Some(key_ty),
        StorageEntryType::Plain(_) => None,
    };
    let params = if let Some(id) = type_id {
        let ty = types.resolve(id).context("invalid metadata")?;
        if let TypeDef::Tuple(_) = ty.type_def {
            type_distributor(params, &ty.type_def, types)?
        } else {
            let json_params = params.as_array().context("expected array")?;
            let params = json_params.iter().next().context("invalid params")?.clone();
            type_distributor(params, &ty.type_def, types)?
        }
    } else {
        vec![]
    };

    let params = set_params_acc_to_storage(params);

    let storage_address = subxt::dynamic::storage(pallet_name, storage_name, params);

    let data = subxt.storage().at_latest().await?.fetch_or_default(&storage_address).await?;

    let serde_val = if data.encoded() == [0] {
        Value::Null
    } else {
        let abc = data.to_value()?;
        scale_to_serde_json(abc.value)?
    };

    Ok(serde_val)
}

fn set_params_acc_to_storage(values: Vec<SubxtValue>) -> Vec<SubxtValue> {
    let mut modified_value = vec![];
    for value in values.clone() {
        if let ValueDef::Composite(inner_val) = value.value.clone() {
            let inner_values = inner_val.into_values();
            for inner_val in inner_values {
                modified_value.push(inner_val);
            }
        } else {
            return values;
        }
    }
    modified_value
}

fn get_type_def(id: u32, types: &PortableRegistry) -> Result<&TypeDef<PortableForm>> {
    let ty = types.resolve(id).context("invalid metadata")?;
    Ok(&ty.type_def)
}

fn type_distributor(
    json_value: Value,
    type_from_pallet: &TypeDef<PortableForm>,
    types: &PortableRegistry,
) -> Result<Vec<SubxtValue>> {
    let mut value_vec = vec![];
    let val = match type_from_pallet {
        TypeDef::Variant(inner_val) => make_variant(&json_value, inner_val, types),
        TypeDef::Composite(inner_val) => make_composite(json_value, inner_val, types),
        TypeDef::Array(inner_val) => make_array(json_value, inner_val),
        TypeDef::Tuple(inner_val) => make_tuple(json_value, inner_val, types),
        TypeDef::Sequence(inner_val) => make_sequence(json_value, inner_val, types),
        TypeDef::Primitive(inner_val) => make_primitive(json_value, inner_val),
        TypeDef::Compact(inner_val) => make_compact(json_value, inner_val),
        TypeDef::BitSequence(inner_val) => make_bit_sequence(json_value, inner_val),
    }?;
    value_vec.push(val);
    Ok(value_vec)
}

fn make_variant(
    json_value: &Value,
    type_from_pallet: &TypeDefVariant<PortableForm>,
    types: &PortableRegistry,
) -> Result<SubxtValue> {
    let variants = &type_from_pallet.variants;
    let mut vec_of_named_data: Vec<(String, SubxtValue)> = vec![];
    let mut vec_of_unnamed_data: Vec<SubxtValue> = vec![];

    let json_key = json_value.as_array().context("invalid params")?[0]
        .as_str()
        .context("invalid params")?;
    let json_value = json_value.as_array().context("invalid params")?[1].clone();

    let fields = variants
        .iter()
        .find(|p| p.name == json_key)
        .context("invalid variant id")?
        .fields
        .clone();
    let is_named = fields.iter().any(|f| f.name.is_some());

    for field in fields {
        let ty_id = field.ty.id;
        let type_def = get_type_def(ty_id, types)?;
        let obtained_result = type_distributor(json_value.clone(), type_def, types);
        if let Ok(obtained_types) = obtained_result {
            if let Some(obtained_type) = obtained_types.into_iter().next() {
                if is_named {
                    vec_of_named_data
                        .push((field.name.context("invalid metadata")?.to_string(), obtained_type));
                } else {
                    vec_of_unnamed_data.push(obtained_type);
                }
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
) -> Result<SubxtValue> {
    let fields = &type_from_pallet.fields;
    let mut vec_of_named_data: Vec<(String, SubxtValue)> = vec![];
    let mut vec_of_unnamed_data: Vec<SubxtValue> = vec![];

    let is_named = fields.iter().any(|f| f.name.is_some());

    match fields.len().cmp(&1) {
        std::cmp::Ordering::Equal => {
            let field = fields[0].clone();
            let ty_id = field.ty.id;
            let type_def = get_type_def(ty_id, types)?;
            let obtained_result = type_distributor(json_value, type_def, types);
            if let Ok(obtained_types) = obtained_result {
                if let Some(obtained_type) = obtained_types.into_iter().next() {
                    if is_named {
                        vec_of_named_data
                            .push((field.name.context("invalid metadata")?, obtained_type));
                    } else {
                        vec_of_unnamed_data.push(obtained_type);
                    }
                }
            }
        },
        std::cmp::Ordering::Greater => {
            let json_value = json_value.as_array().context("invalid params")?;
            for (value_received, field) in json_value.iter().zip(fields) {
                let ty_id = field.ty.id;
                let type_def = get_type_def(ty_id, types)?;
                let obtained_result = type_distributor(value_received.clone(), type_def, types);
                if let Ok(obtained_types) = obtained_result {
                    if let Some(obtained_type) = obtained_types.into_iter().next() {
                        if is_named {
                            vec_of_named_data.push((
                                field.name.as_ref().context("invalid metadata")?.clone(),
                                obtained_type,
                            ));
                        } else {
                            vec_of_unnamed_data.push(obtained_type);
                        }
                    }
                }
            }
        },
        std::cmp::Ordering::Less => {
            //keep the vector empty
        },
    }

    if is_named {
        Ok(SubxtValue::named_composite(vec_of_named_data))
    } else {
        Ok(SubxtValue::unnamed_composite(vec_of_unnamed_data))
    }
}

fn make_sequence(
    json_value: Value,
    type_from_pallet: &TypeDefSequence<PortableForm>,
    types: &PortableRegistry,
) -> Result<SubxtValue> {
    let mut vec_of_data = vec![];
    let id = type_from_pallet.type_param.id;
    let type_def = get_type_def(id, types)?;
    let converted_type = type_distributor(json_value, type_def, types)?;
    for val in converted_type {
        vec_of_data.push(val);
    }

    let return_val = SubxtValue::unnamed_composite(vec_of_data);
    Ok(return_val)
}

fn make_array(
    json_value: Value,
    _type_from_pallet: &TypeDefArray<PortableForm>,
) -> Result<SubxtValue> {
    if let Value::Array(val) = json_value {
        let mut vec_value = vec![];
        for value in val {
            let str_number = value.to_string();
            let parsed_number = str_number.parse::<u8>()?;
            vec_value.push(parsed_number);
        }
        let referenced_vec = &vec_value;
        let bytes_data: &[u8] = referenced_vec;
        return Ok(SubxtValue::from_bytes(bytes_data));
    }
    anyhow::bail!("expected array");
}

fn make_tuple(
    json_value: Value,
    type_from_pallet: &TypeDefTuple<PortableForm>,
    types: &PortableRegistry,
) -> Result<SubxtValue> {
    let mut values_vec = vec![];
    let fields = &type_from_pallet.fields;
    if let Value::Array(val) = json_value {
        for (value, field) in val.iter().zip(fields) {
            let ty_id = field.id;
            let type_def = get_type_def(ty_id, types)?;
            let converted_vals = type_distributor(value.clone(), type_def, types)?;
            for converted_val in converted_vals {
                values_vec.push(converted_val);
            }
        }
    }
    Ok(SubxtValue::unnamed_composite(values_vec))
}

fn make_primitive(json_value: Value, _type_from_pallet: &TypeDefPrimitive) -> Result<SubxtValue> {
    match json_value {
        Value::Bool(val) => Ok(SubxtValue::bool(val)),
        Value::Number(val) => {
            let number_string = val.to_string();
            let number_i128 = number_string.parse::<u128>()?;
            Ok(SubxtValue::u128(number_i128))
        },
        Value::String(val) => Ok(SubxtValue::string(val)),
        _ => anyhow::bail!("expected bool number or string"),
    }
}

fn make_compact(
    json_value: Value,
    _type_from_pallet: &TypeDefCompact<PortableForm>,
) -> Result<SubxtValue> {
    match json_value {
        Value::Number(val) => {
            let number_string = val.to_string();
            let number_i128 = number_string.parse::<u128>()?;
            Ok(SubxtValue::u128(number_i128))
        },
        _ => anyhow::bail!("expected number"),
    }
}

fn make_bit_sequence(
    json_value: Value,
    _type_from_pallet: &TypeDefBitSequence<PortableForm>,
) -> Result<SubxtValue> {
    let mut bits_array = BitSequence::new();
    if let Value::Array(values) = json_value {
        for value in values {
            match value {
                Value::Bool(val) => bits_array.push(val),
                Value::Number(val) => {
                    let number = val.as_u64().context("invalid params")?;
                    bits_array.push(number != 0);
                },
                _ => anyhow::bail!("expected bit sequence"),
            }
        }
    }
    Ok(SubxtValue::bit_sequence(bits_array))
}

fn scale_to_serde_json(data: ValueDef<TypeId>) -> Result<SerdeValue> {
    match data {
        scale_value::ValueDef::Composite(val) => match val {
            scale_value::Composite::Named(named_composite) => {
                let mut map = Map::new();
                for (key, value) in named_composite {
                    map.insert(key, scale_to_serde_json(value.value)?);
                }
                Ok(SerdeValue::Object(map))
            },
            scale_value::Composite::Unnamed(val) => {
                let mut vec_of_array = vec![];
                for value in val {
                    vec_of_array.push(scale_to_serde_json(value.value)?);
                }
                Ok(SerdeValue::Array(vec_of_array))
            },
        },
        scale_value::ValueDef::Variant(val) => {
            if val.values.is_empty() {
                Ok(SerdeValue::String(val.name))
            } else {
                let mut map = Map::new();
                map.insert(val.name, scale_to_serde_json(val.values.into())?);
                Ok(SerdeValue::Object(map))
            }
        },
        scale_value::ValueDef::BitSequence(val) => {
            let mut vec_of_array = vec![];
            for i in val {
                vec_of_array.push(SerdeValue::Bool(i));
            }
            Ok(SerdeValue::Array(vec_of_array))
        },
        scale_value::ValueDef::Primitive(val) => match val {
            scale_value::Primitive::Bool(val) => Ok(SerdeValue::Bool(val)),
            scale_value::Primitive::Char(val) => Ok(SerdeValue::String(val.to_string())),
            scale_value::Primitive::String(val) => Ok(SerdeValue::String(val)),
            _ => Ok(serde_json::to_value(val.clone())?),
        },
    }
}

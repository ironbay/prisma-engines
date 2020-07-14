use crate::misc_helpers::{
    calculate_backrelation_field, calculate_index, calculate_many_to_many_field, calculate_relation_field,
    calculate_scalar_field, is_migration_table, is_prisma_1_point_0_join_table, is_prisma_1_point_1_or_2_join_table,
    is_relay_table,
};
use crate::version_checker::VersionChecker;
use crate::SqlError;
use datamodel::{dml, Datamodel, Field, FieldType, Model};
use sql_schema_describer::SqlSchema;
use tracing::debug;

pub fn introspect(
    schema: &SqlSchema,
    version_check: &mut VersionChecker,
    data_model: &mut Datamodel,
) -> Result<(), SqlError> {
    for table in schema
        .tables
        .iter()
        .filter(|table| !is_migration_table(&table))
        .filter(|table| !is_prisma_1_point_1_or_2_join_table(&table))
        .filter(|table| !is_prisma_1_point_0_join_table(&table))
        .filter(|table| !is_relay_table(&table))
    {
        debug!("Calculating model: {}", table.name);
        let mut model = Model::new(table.name.clone(), None);

        for column in &table.columns {
            version_check.check_column_for_type_and_default_value(&column);
            let field = calculate_scalar_field(&table, &column);
            model.add_field(Field::ScalarField(field));
        }

        let mut foreign_keys_copy = table.foreign_keys.clone();
        let model_copy = model.clone();
        foreign_keys_copy.clear_duplicates();

        for foreign_key in foreign_keys_copy.iter().filter(|fk| {
            !fk.columns.iter().any(|c| {
                matches!(
                    model_copy.find_scalar_field(c).unwrap().field_type,
                    FieldType::Unsupported(_)
                )
            })
        }) {
            version_check.has_inline_relations(table);
            version_check.uses_on_delete(foreign_key, table);
            model.add_field(Field::RelationField(calculate_relation_field(
                schema,
                table,
                foreign_key,
            )?));
        }

        for index in table
            .indices
            .iter()
            .filter(|i| !(i.columns.len() == 1 && i.is_unique()))
        {
            model.add_index(calculate_index(index));
        }

        if table.primary_key_columns().len() > 1 {
            model.id_fields = table.primary_key_columns();
        }

        version_check.always_has_created_at_updated_at(table, &model);
        version_check.has_p1_compatible_primary_key_column(table);

        data_model.add_model(model);
    }

    for e in schema.enums.iter() {
        data_model.add_enum(dml::Enum::new(
            &e.name,
            e.values.iter().map(|v| dml::EnumValue::new(v)).collect(),
        ));
    }

    let mut fields_to_be_added = Vec::new();

    // add backrelation fields
    for model in data_model.models() {
        for relation_field in model.relation_fields() {
            let relation_info = &relation_field.relation_info;
            if data_model
                .find_related_field_for_info(&relation_info, &relation_field.name)
                .is_none()
            {
                let other_model = data_model.find_model(relation_info.to.as_str()).unwrap();
                let field = calculate_backrelation_field(schema, model, other_model, relation_field, relation_info)?;

                fields_to_be_added.push((other_model.name.clone(), field));
            }
        }
    }

    // add prisma many to many relation fields
    for table in schema
        .tables
        .iter()
        .filter(|table| is_prisma_1_point_1_or_2_join_table(&table) || is_prisma_1_point_0_join_table(&table))
    {
        if let (Some(f), Some(s)) = (table.foreign_keys.get(0), table.foreign_keys.get(1)) {
            let is_self_relation = f.referenced_table == s.referenced_table;

            fields_to_be_added.push((
                s.referenced_table.clone(),
                calculate_many_to_many_field(f, table.name[1..].to_string(), is_self_relation),
            ));
            fields_to_be_added.push((
                f.referenced_table.clone(),
                calculate_many_to_many_field(s, table.name[1..].to_string(), is_self_relation),
            ));
        }
    }

    for (model, field) in fields_to_be_added {
        data_model.find_model_mut(&model).add_field(Field::RelationField(field));
    }

    Ok(())
}

trait Dedup<T: PartialEq + Clone> {
    fn clear_duplicates(&mut self);
}

impl<T: PartialEq + Clone> Dedup<T> for Vec<T> {
    fn clear_duplicates(&mut self) {
        let mut already_seen = vec![];
        self.retain(|item| match already_seen.contains(item) {
            true => false,
            _ => {
                already_seen.push(item.clone());
                true
            }
        })
    }
}

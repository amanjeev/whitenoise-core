use crate::errors::*;

use crate::{proto, base, Warnable};

use crate::components::{Component};
use crate::base::{Value, ValueProperties, DataType, IndexKey, ArrayProperties, DataframeProperties};
use crate::utilities::prepend;
use indexmap::map::IndexMap;
use crate::components::transforms::propagate_binary_group_id;


impl Component for proto::TheilSen {
    fn propagate_property(
        &self,
        _privacy_definition: &Option<proto::PrivacyDefinition>,
        _public_arguments: IndexMap<base::IndexKey, &Value>,
        properties: base::NodeProperties,
        node_id: u32,
    ) -> Result<Warnable<ValueProperties>> {

        let num_records = Some(0);

        let mut output_properties = ArrayProperties {
            num_records,
            num_columns: Some(2),
            nullity: false,
            releasable: false, // x.releasable && y.releasable,
            c_stability: Vec::new(),
            aggregator: None,
            nature: None,
            data_type: DataType::Float,
            dataset_id: None,
            is_not_empty: true,
            dimensionality: Some(1),

            // TODO
            group_id: Vec::new(),
        };

        let data_property_x = properties.get::<IndexKey>(&"data_x".into())
            .ok_or("data x: missing")?.array()
            .map_err(prepend("data x:"))?.clone();

        let data_property_y = properties.get::<IndexKey>(&"data_y".into())
            .ok_or("data y: missing")?.array()
            .map_err(prepend("data y:"))?.clone();

        if !data_property_x.releasable {
            data_property_x.assert_is_not_aggregated()?;
        }
        if !data_property_y.releasable {
            data_property_y.assert_is_not_aggregated()?;
        }
        data_property_x.assert_is_not_empty()?;
        data_property_y.assert_is_not_empty()?;


        if data_property_x.data_type != DataType::Float {
            return Err("data x: atomic type must be float".into());
        }

        if data_property_y.data_type != DataType::Float {
            return Err("data y: atomic type must be float".into());
        }

        if data_property_x.num_records != data_property_y.num_records {
            return Err("data x and data y: must be same length".into());
        }

        let num_records = match self.implementation.to_lowercase().as_str() {
            "theil-sen" => data_property_x.num_records()?.pow(2),
            "theil-sen-k-match" => ((self.k as i64) * data_property_x.num_records()? / 2) as i64,
             _ => return Err("Invalid implementation passed. \
                     Valid values are theil-sen and theil-sen-k-match".into())
        };
        let num_records = 1;
        output_properties.num_records = Some(num_records);
        output_properties.dataset_id = Some(node_id as i64);
        output_properties.releasable = data_property_x.releasable && data_property_y.releasable;
        output_properties.group_id = propagate_binary_group_id(&data_property_x, &data_property_y)?;
        output_properties.c_stability = data_property_x.c_stability.iter().zip(data_property_y.c_stability.iter()).map(|(l, r)| l * r).collect();

        let result: ValueProperties = output_properties.into();

        Ok(ValueProperties::Dataframe(DataframeProperties {
            children: indexmap![IndexKey::from("slope") => result.clone() as ValueProperties,
                                IndexKey::from("intercept") => result]
        }).into())
    }
}
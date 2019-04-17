use connector::{QueryArguments, ScalarListValues};
use prisma_models::{GraphqlId, ManyNodes, PrismaValue, SelectedFields, SelectedScalarField, SingleNode};

#[derive(Debug)]
pub enum ReadQueryResult {
    Single(SingleReadQueryResult),
    Many(ManyReadQueryResults),
}

#[derive(Debug)]
pub struct SingleReadQueryResult {
    pub name: String,
    pub fields: Vec<String>,

    /// Scalar field results
    pub scalars: Option<SingleNode>,

    /// Nested queries results
    pub nested: Vec<ReadQueryResult>,

    /// Scalar list results, field names mapped to their results
    pub lists: Vec<(String, Vec<ScalarListValues>)>,

    /// Used for filtering implicit fields in result records
    pub selected_fields: SelectedFields,
}

#[derive(Debug)]
pub struct ManyReadQueryResults {
    pub name: String,
    pub fields: Vec<String>,

    /// Scalar field results
    pub scalars: ManyNodes,

    /// Nested queries results
    pub nested: Vec<ReadQueryResult>,

    /// Scalar list results, field names mapped to their results
    pub lists: Vec<(String, Vec<ScalarListValues>)>,

    /// Required for result processing
    pub query_arguments: QueryArguments,

    /// Used for filtering implicit fields in result records
    pub selected_fields: SelectedFields,
}

// Q: Best pattern here? Mix of in place mutation and recreating result
impl SingleReadQueryResult {
    /// Returns the implicitly added fields
    pub fn get_implicit_fields(&self) -> Vec<&SelectedScalarField> {
        self.selected_fields.get_implicit_fields()
    }

    /// Get the ID from a record
    pub fn find_id(&self) -> Option<&GraphqlId> {
        let id_position: usize = self
            .scalars
            .as_ref()
            .map_or(None, |r| r.field_names.iter().position(|name| name == "id"))?;

        self.scalars.as_ref().map_or(None, |r| {
            r.node.values.get(id_position).map(|pv| match pv {
                PrismaValue::GraphqlId(id) => Some(id),
                _ => None,
            })?
        })
    }
}

impl ManyReadQueryResults {
    /// Returns the implicitly added fields
    pub fn get_implicit_fields(&self) -> Vec<&SelectedScalarField> {
        self.selected_fields.get_implicit_fields()
    }

    /// Note: At the moment, this is only to strip the excess records added to by the database query layer
    pub fn add_pagination_info(self) -> () {
        let reversed = self.query_arguments.last.is_some();
        if reversed {
            self.scalars.reverse();
        }

        // (queryArguments.first, queryArguments.last) match {
        //   case (Some(f), _) if items.size > f => ResolverResult(items.dropRight(1), hasPreviousPage = false, hasNextPage = true, parentModelId = parentModelId)
        //   case (_, Some(l)) if items.size > l => ResolverResult(items.tail, hasPreviousPage = true, hasNextPage = false, parentModelId = parentModelId)
        //   case _                              => ResolverResult(items, hasPreviousPage = false, hasNextPage = false, parentModelId = parentModelId)
        // }
    }

    /// Get all IDs from a query result
    pub fn find_ids(&self) -> Option<Vec<&GraphqlId>> {
        let id_position: usize = self.scalars.field_names.iter().position(|name| name == "id")?;
        self.scalars
            .nodes
            .iter()
            .map(|node| node.values.get(id_position))
            .map(|pv| match pv {
                Some(PrismaValue::GraphqlId(id)) => Some(id),
                _ => None,
            })
            .collect()
    }
}
use {
    super::{context::RowContext, evaluate::evaluate},
    crate::{
        ast::{Aggregate, Expr},
        data::Value,
        result::Result,
        store::GStore,
    },
    im_rc::HashMap,
    std::rc::Rc,
};

pub struct Filter<'a> {
    storage: &'a dyn GStore,
    where_clause: Option<&'a Expr>,
    context: Option<Rc<RowContext<'a>>>,
    aggregated: Option<Rc<HashMap<&'a Aggregate, Value>>>,
}

impl<'a> Filter<'a> {
    pub fn new(
        storage: &'a dyn GStore,
        where_clause: Option<&'a Expr>,
        context: Option<Rc<RowContext<'a>>>,
        aggregated: Option<Rc<HashMap<&'a Aggregate, Value>>>,
    ) -> Self {
        Self {
            storage,
            where_clause,
            context,
            aggregated,
        }
    }

    pub async fn check(&self, blend_context: Rc<RowContext<'a>>) -> Result<bool> {
        match self.where_clause {
            Some(expr) => {
                let context = match &self.context {
                    Some(context) => Rc::new(RowContext::concat(blend_context, Rc::clone(context))),
                    None => blend_context,
                };
                let context = Some(context);
                let aggregated = self.aggregated.as_ref().map(Rc::clone);

                check_expr(self.storage, context, aggregated, expr).await
            }
            None => Ok(true),
        }
    }
}

pub async fn check_expr<'a>(
    storage: &'a dyn GStore,
    context: Option<Rc<RowContext<'a>>>,
    aggregated: Option<Rc<HashMap<&'a Aggregate, Value>>>,
    expr: &'a Expr,
) -> Result<bool> {
    evaluate(storage, context, aggregated, expr)
        .await
        .map(|evaluated| evaluated.try_into())?
}

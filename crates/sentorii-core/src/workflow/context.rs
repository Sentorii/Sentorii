use sentorii_config::Config;
use sentorii_contracts::context::{Context, ContextBuilder};

pub trait ContextProvider {
    fn to_context(&self) -> Context;
}

impl ContextProvider for Config {
    fn to_context(&self) -> Context {
        ContextBuilder::new()
            .with_main(&self.branching.main)
            .with_develop(&self.branching.develop)
            .build()
    }
}

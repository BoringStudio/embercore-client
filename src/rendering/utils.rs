use super::prelude::*;

pub trait IntoDescriptorSet {
    fn into_descriptor_set(
        self,
        pipeline: &(dyn GraphicsPipelineAbstract + Send + Sync),
    ) -> Arc<dyn DescriptorSet + Send + Sync>;
}

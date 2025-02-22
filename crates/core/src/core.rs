use std::ops::Deref;

use crate::config::Config;
use crate::source::Source;
use crate::v8_sys::{
    local_string_from_string, string_from_local_string, v8_context_get_isolate, V8Context, V8Source,
};

pub(crate) unsafe fn process_script(
    config: &Config,
    v8_context: *const V8Context,
    v8_source: *mut V8Source,
) {
    let isolate = v8_context_get_isolate(v8_context);
    let resource_name = string_from_local_string(isolate, (*v8_source)._resource_name);
    let source_string = string_from_local_string(isolate, (*v8_source)._source_string);
    let mut source = Source {
        resource_name,
        source_string,
    };
    config.rules.iter().for_each(|rule_item| {
        let (rule_name, rule) = rule_item;
        let is_match = &rule.matcher.deref().matches(&source);
        if *is_match {
            println!(
                "[*] Rule {} matched in {}",
                rule_name, &source.resource_name
            );
            let processors = &rule.processors;
            processors.iter().for_each(|processor_item| {
                let processor = processor_item.deref();
                let result = processor.process(&mut source);
                if result.is_err() {
                    println!(
                        "[!] Processor {:#?} process failed: {}",
                        processor,
                        result.err().unwrap()
                    );
                };
            });
        }
    });
    let processed_source_string = source.source_string.as_str();
    let processed_local_string =
        local_string_from_string(isolate, processed_source_string.to_string());
    (*v8_source)._source_string = processed_local_string;
}

use log::{debug, trace};
use std::io::{self, prelude::*};

use crate::collapse::common::{self, CollapsePrivate, Occurrences};

/// A stack collapser for the output of `xctrace record -t time && xctrace export --input PATH/TO/TRACE/FILE.trace --xpath '/trace-toc/run[@number="1"]/data/table[@schema="time-sample"]'`.
pub struct Folder;

impl CollapsePrivate for Folder {
    fn pre_process<R>(&mut self, reader: &mut R, occurrences: &mut Occurrences) -> io::Result<()>
    where
        R: io::BufRead,
    {
        let mut line = Vec::new();
        // FIXME the first row is appened to the last header, and is ignored for now
        Ok(for _ in 0..3 {
            reader.read_until(b'\n', &mut line)?;
        })
    }

    // every line is a complete stack
    fn would_end_stack(&mut self, _: &[u8]) -> bool {
        true
    }

    fn clone_and_reset_stack_context(&self) -> Self {
        Self
    }

    fn is_applicable(&mut self, input: &str) -> Option<bool> {
        unimplemented!()
    }

    fn nstacks_per_job(&self) -> usize {
        1 // DEBUG
    }

    fn set_nstacks_per_job(&mut self, n: usize) {
        // DEBUG
    }

    fn nthreads(&self) -> usize {
        1 // DEBUG
    }

    fn set_nthreads(&mut self, n: usize) {
        // DEBUG
    }

    fn collapse_single_threaded<R>(
        &mut self,
        reader: R,
        occurrences: &mut Occurrences,
    ) -> io::Result<()>
    where
        R: io::BufRead,
    {
        for line_res in reader.lines() {
            let line = line_res?;
            let doc = match roxmltree::Document::parse(&line) {
                Ok(doc) => doc,
                Err(e) => {
                    // last line of the file
                    if line == "</node></trace-query-result>" {
                        return Ok(());
                    } else {
                        return invalid_data_error!(
                            "Failed to parse XML row: {}, error: {}",
                            line,
                            e
                        );
                    }
                }
            };
            trace!("{doc:?}");
            trace!("{:?}", doc.descendants());
            let backtrace = doc
                .descendants()
                .find(|n| n.has_tag_name("backtrace"))
                .ok_or(invalid_data!("Unable find backtrace"))?;
            trace!("{backtrace:?}");
            if let Some(function_name) = backtrace
                .attribute("fmt")
                .and_then(|s| s.split_once(' '))
                .map(|s| s.0)
            {
                debug!("function_name: {function_name:?}");
                // for some unknown reasons there are sometimes multiple stacks, for now keep the longest
                let mut longest_stack_opt = None;
                for stack in backtrace
                    .descendants()
                    .filter(|n| n.has_tag_name("text-addresses"))
                {
                    if let Some(lstack) = longest_stack_opt {
                        if node_len(stack) > node_len(lstack) {
                            longest_stack_opt = Some(stack)
                        }
                    } else {
                        longest_stack_opt = Some(stack)
                    }
                }
                // in case there is no stack we pass, so sure if it's sound behavior
                debug!("longest_stack_opt = {:?}", longest_stack_opt);
                if let Some(longest_stack) = longest_stack_opt.and_then(|n| n.text()) {
                    let stack_str = longest_stack.replace(" ", ";") + ";" + function_name;
                    occurrences.insert_or_add(stack_str, 1);
                }
            }
        }
        debug!("occurrences: {occurrences:?}");
        Ok(())
    }
}

#[inline]
fn node_len(n: roxmltree::Node) -> usize {
    n.text()
        .map(|text| text.len()).unwrap_or(0)
}

impl Default for Folder {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn fuck() {
        init();
        let row = r#"<row><sample-time id="1407" fmt="00:01.077.455">1077455833</sample-time><thread ref="2"/><process ref="4"/><core ref="7"/><thread-state ref="8"/><weight ref="9"/><backtrace id="1408" fmt="retroboard::retroboard::RetroBoard::is_safe::h10636cc78360298f ← (11 other frames)"><process ref="4"/><text-addresses ref="177"/><process ref="4"/><text-addresses id="1409" fmt="frag 2078">4331406716 4331403404 4331396476 4331396516 4331397392 4331396636 4331396660 4331486836 4331398140 4336799884</text-addresses></backtrace></row>
"#.as_bytes();
        let mut folder = Folder::default();
        folder
            .collapse_single_threaded(row, &mut Occurrences::new(1))
            .unwrap();
        panic!("t");
    }
}

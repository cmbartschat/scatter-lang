use crate::lang::{Block, Loop, Term};

pub fn enumerate_captures<F>(block: &Block, callback: &mut F)
where
    F: FnMut(&str),
{
    for term in &block.terms {
        match term {
            Term::String(..)
            | Term::Number(..)
            | Term::Bool(..)
            | Term::Name(..)
            | Term::Address(..) => {}
            Term::Capture(z, _) => callback(z),
            Term::Branch(branch) => {
                for (condition, block) in &branch.arms {
                    enumerate_captures(condition, callback);
                    enumerate_captures(block, callback);
                }
            }
            Term::Loop(Loop {
                pre_condition,
                body,
                post_condition,
            }) => {
                if let Some(pre_condition) = &pre_condition {
                    enumerate_captures(pre_condition, callback);
                }
                enumerate_captures(body, callback);
                if let Some(post_condition) = &post_condition {
                    enumerate_captures(post_condition, callback);
                }
            }
        }
    }
}

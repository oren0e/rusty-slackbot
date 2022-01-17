use slack_morphism::prelude::*;

#[derive(Debug, Clone)]
pub struct CodeReplyTemplate<'a> {
    pub share_link: &'a str,
    pub stdout: String,
    pub stderr: String,
}

impl<'a> CodeReplyTemplate<'a> {
    pub fn new(share_link: &'a str, stdout: String, stderr: String) -> Self {
        Self {
            share_link,
            stdout,
            stderr,
        }
    }
}

impl<'a> SlackMessageTemplate for CodeReplyTemplate<'a> {
    fn render_template(&self) -> SlackMessageContent {
        SlackMessageContent::new()
            .with_text("Executing...".to_string())
            .with_blocks(slack_blocks![
                some_into(SlackHeaderBlock::new(SlackBlockText::Plain(
                    SlackBlockPlainText::new("Rust Playground".to_string())
                ))),
                some_into(SlackActionsBlock::new(slack_blocks![some_into(
                    SlackBlockButtonElement::new(
                        SlackActionId("button-action".to_string()),
                        pt!("Code")
                    )
                )])),
                some_into(SlackContextBlock::new(vec![
                    SlackContextBlockElement::Plain(SlackBlockPlainText::new("Stdout".to_string()))
                ])),
                some_into(SlackSectionBlock::new().with_text(md!("```{}```", self.stdout))),
                some_into(SlackDividerBlock::new()),
                some_into(SlackContextBlock::new(vec![
                    SlackContextBlockElement::Plain(SlackBlockPlainText::new("Stderr".to_string()))
                ])),
                some_into(SlackSectionBlock::new().with_text(md!("```{}```", self.stderr)))
            ])
    }
}

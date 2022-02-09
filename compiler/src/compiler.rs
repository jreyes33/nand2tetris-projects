use crate::Result;
use tree_sitter::{Node, TreeCursor};
use vm_translator::parser::{Command, Segment};

pub struct Compiler<'c> {
    source: &'c str,
    cursor: TreeCursor<'c>,
    commands: Vec<Command<'c>>,
}

impl<'c> Compiler<'c> {
    pub fn new(source: &'c str, cursor: TreeCursor<'c>) -> Self {
        Self {
            source,
            cursor,
            commands: vec![],
        }
    }

    pub fn compile(&mut self) -> Result<&[Command]> {
        // TODO: maybe use kind_id instead?
        // Note that the ID could change if the Jack grammar changes.
        match self.node().kind() {
            "class_declaration" => self.class_declaration()?,
            k => return Err(format!("unsupported {k} at top level").into()),
        }
        Ok(&self.commands)
    }

    fn class_declaration(&mut self) -> Result<()> {
        self.goto_first_child();
        self.advance_to("class_body")?;
        self.goto_first_child();
        loop {
            let node = self.node();
            if node.is_named() && !node.is_extra() {
                self.class_member()?;
            }
            if !self.goto_next_sibling() {
                break;
            }
        }
        self.goto_parent();
        self.goto_parent();
        Ok(())
    }

    fn class_member(&mut self) -> Result<()> {
        match self.node().kind() {
            "function_declaration" => self.function_declaration(),
            k => Err(format!("unsupported {k} class member").into()),
        }
    }

    fn function_declaration(&mut self) -> Result<()> {
        self.goto_first_child();
        // self.advance_to("return_type")?;
        // dbg!(self.text());
        self.advance_to("identifier")?;
        let name = self.text();
        self.advance_to("parameter_list")?;
        self.goto_first_child();
        let mut param_count = 0;
        loop {
            if self.node().kind() == "parameter" {
                param_count += 1;
            }
            if !self.goto_next_sibling() {
                break;
            }
        }
        self.add(Command::Function(name, param_count));
        self.goto_parent();
        self.advance_to("subroutine_body")?;
        self.subroutine_body()?;
        self.goto_parent();
        Ok(())
    }

    fn subroutine_body(&mut self) -> Result<()> {
        // TODO: compile var_declarations.
        self.goto_first_child();
        self.advance_to("statements")?;
        loop {
            let node = self.node();
            if node.is_named() && !node.is_extra() {
                self.statement()?;
            }
            if !self.goto_next_sibling() {
                break;
            }
        }
        self.goto_parent();
        Ok(())
    }

    fn statement(&mut self) -> Result<()> {
        self.goto_first_child();
        let result = match self.node().kind() {
            "do_statement" => self.do_statement(),
            k => Err(format!("unsupported {k} statement").into()),
        };
        self.goto_parent();
        result
    }

    fn do_statement(&mut self) -> Result<()> {
        self.goto_first_child();
        self.advance_to("subroutine_call")?;
        self.subroutine_call()?;
        self.goto_parent();
        Ok(())
    }

    fn subroutine_call(&mut self) -> Result<()> {
        self.goto_first_child();
        let name = self.text();
        self.advance_to("expression_list")?;
        self.goto_first_child();
        loop {
            let node = self.node();
            if node.is_named() && !node.is_extra() {
                self.expression()?;
            }
            if !self.goto_next_sibling() {
                break;
            }
        }
        self.goto_parent();
        let arg_count = 1; // TODO: calculate.
        self.add(Command::Call(name, arg_count));
        self.goto_parent();
        Ok(())
    }

    fn expression(&mut self) -> Result<()> {
        self.goto_first_child();
        self.term()?;
        self.goto_parent();
        todo!("continue here, need to walk subsequent ops and terms");
    }

    fn term(&mut self) -> Result<()> {
        // TODO: dry this up with closures maybe.
        self.goto_first_child();
        match self.node().kind() {
            "integer_constant" => self.add(Command::Push(Segment::Constant, self.text().parse()?)),
            k => todo!("handle {k} term"),
        }
        self.goto_parent();
        Ok(())
    }

    fn node(&self) -> Node<'c> {
        self.cursor.node()
    }

    fn text(&self) -> &'c str {
        self.node()
            .utf8_text(self.source.as_bytes())
            .expect("source was already valid UTF-8")
    }

    fn add(&mut self, command: Command<'c>) {
        self.commands.push(command);
    }

    fn advance_to(&mut self, kind: &str) -> Result<()> {
        loop {
            if self.node().kind() == kind {
                return Ok(());
            }
            if !self.goto_next_sibling() {
                return Err(format!("expected {kind}").into());
            }
        }
    }

    fn goto_first_child(&mut self) -> bool {
        self.cursor.goto_first_child()
    }

    fn goto_next_sibling(&mut self) -> bool {
        self.cursor.goto_next_sibling()
    }

    fn goto_parent(&mut self) -> bool {
        self.cursor.goto_parent()
    }

    #[allow(unused)]
    fn dbg_node(&self) {
        let node = self.node();
        println!(
            "Source: {}\nNode:   {:?}\nTree:   {}",
            self.text(),
            node,
            node.to_sexp(),
        );
    }
}

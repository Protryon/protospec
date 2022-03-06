use super::*;

impl Context {
    pub fn encode_var_ref(&mut self, type_: &TypeRef, target: Target, source: usize) {
        if let Type::Foreign(f) = &*type_.target.type_.borrow() {
            let mut function_args = vec![];
            for expr in &type_.arguments {
                let r = self.alloc_register();
                self.instructions.push(Instruction::Eval(r, expr.clone()));
                function_args.push(r);
            }
            self.instructions.push(Instruction::EncodeForeign(
                target,
                source,
                f.clone(),
                function_args,
            ));
        } else {
            let mut args = vec![];
            for arg in type_.arguments.iter() {
                let r = self.alloc_register();
                self.instructions.push(Instruction::Eval(r, arg.clone()));
                args.push(r);
            }    
            self.instructions
                .push(Instruction::EncodeRef(target, source, args));
        }
    }
}
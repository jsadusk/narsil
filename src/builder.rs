use std::ops::Mul;
use std::cell::{Ref, RefCell};
use std::rc::Rc;

#[derive(Fail, Debug)]
pub enum BuilderError {
    #[fail(display = "Tried to get() a result that has not been calculated")]
    GetNotCalculated,
}

type BuilderResult<T> = Result<T, BuilderError>;

#[derive(Debug, Copy, Clone)]
struct Term(usize);

struct TypedTerm<ResultType> {
    term: Term,
    result: Rc<RefCell<Option<ResultType>>>
}

impl<ResultType> TypedTerm<ResultType> {
    fn get(&self) -> BuilderResult<Ref<Option<ResultType>>> {
        if self.result.as_ref().borrow().is_none() {
            Err(BuilderError::GetNotCalculated)
        } else {
            Ok(self.result.as_ref().borrow())
        }
    }

    fn term(&self) -> Term {
        self.term
    }
}

type Terms = Vec<Term>;

pub trait Expression<ValueType> {
    fn terms(&self) -> Terms;
    fn eval(&self) -> BuilderResult<ValueType>;
}

struct TypedExpressionResult<ResultType, Expr: Expression<ResultType>> {
    expr: Expr,
    result: Rc<RefCell<Option<ResultType>>>
}

/*impl<'a, ResultType> TypedExpressionResult<'a, ResultType> {
    fn apply<V>(&'a self, term: &mut TypedTerm<'a, ResultType>) {
        
        match self {
            Self::Result(value) => {
                *term = TypedTerm::<'a, ResultType>::Result(&value);
            },
            Self::Expr(expr) => panic!("Cant apply a term that hasn't been eval'd")
        }
    }

}*/

trait ExpressionResult {
    fn evaluated(&self) -> bool;
    fn terms(&self) -> Terms;
    fn eval(&mut self);
}

impl<ResultType, Expr: Expression<ResultType>> ExpressionResult for TypedExpressionResult<ResultType, Expr> {
    fn terms(&self) -> Terms {
        self.expr.terms()
    }

    fn eval(&mut self) {
        match *self.result.borrow() {
            None => {let _ = self.result.replace(Some(self.expr.eval().unwrap())); },
            Some(_) => panic!("Can't get terms for already evaluated expression")
        }
    }

    fn evaluated(&self) -> bool {
        match *self.result.borrow() {
            Some(_) => true,
            None => false
        }
    }
 }

struct Builder {
     terms: Vec<Box<dyn ExpressionResult>>
}

impl Builder {

    fn new() -> Builder {
        Builder { terms: Vec::new() }
    }
    
    fn eval_term(&mut self, term: Term) {
        if !self.terms[term.0].evaluated() {
            for subterm in self.terms[term.0].terms() {
                self.eval_term(subterm);
            }

            self.terms[term.0].eval();
        }
    }

    fn eval<'a, ValueType>(&mut self, term: &'a TypedTerm<ValueType>) -> BuilderResult<Ref<'a, Option<ValueType>>> {
        self.eval_term(term.term());
        term.get()
    }

    fn term<ValueType: 'static, Expr: Expression<ValueType> + 'static>(&mut self, expr: Expr) -> TypedTerm<ValueType> {
        let result = Rc::new(RefCell::<Option<ValueType>>::new(None));
        
        self.terms.push(
            Box::new(
                TypedExpressionResult::<ValueType, Expr> {
                    expr: expr, result: result.clone()}));;

        TypedTerm { term: Term(self.terms.len() - 1),
                    result: result.clone()}
    }
}

/*impl<ValueType, ExpressionType: Expression<ValueType>> Builder<ValueType, ExpressionType> {
    fn get(&mut self) -> ValueType {
        match self.result {
            Some(result) => result,
            None => {
                for term in &mut self.expr.terms() {
                    term.get();
                }
                
                let result = self.expr.eval();
                result.unwrap()
            }
        }
    }
}*/

struct Value<ValueType: Clone> {
    val : ValueType
}

impl<ValueType: Clone> Expression<ValueType> for Value<ValueType> {
    fn eval(&self) -> BuilderResult<ValueType> {
        Ok(self.val.clone())
    }

    fn terms(&self) -> Vec<Term> { Vec::<Term>::new() }
}

struct Multiply<ValueType: Mul + Copy> {
    operand: TypedTerm<ValueType>,
    factor: ValueType
}

impl<ValueType: Mul + Copy> Expression<ValueType::Output> for Multiply<ValueType> {
    fn terms(&self) -> Terms {
        vec!(self.operand.term())
    }
    
    fn eval(&self) -> BuilderResult<ValueType::Output> {
        let result = self.operand.get()?.unwrap() * self.factor;
        Ok(result)
    }
}

mod tests {
    use super::*;

    fn two_term() {
        let mut builder = Builder::new();
        
        let term1 = builder.term(Value::<i32>{ val: 5 });
        let term2 = builder.term(Multiply::<i32>{ operand: term1, factor: 2 });
        assert_eq!(builder.eval(&term2).unwrap().unwrap(), 10);
    }
}

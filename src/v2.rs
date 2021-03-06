pub enum Step<M, O, C> {
    NotReady(M, O),
    Done(C),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
// könnte man auch weglassen und `try_next_state` könnte im fehlerfall direkt C zurückgeben
pub struct UnexpectedEndOfStateMachine<T>(T);

impl<M, O, C> Step<M, O, C> {
    pub fn try_next_state(self) -> Result<(M, O), UnexpectedEndOfStateMachine<C>> {
        match self {
            Step::Done(val) => Err(UnexpectedEndOfStateMachine(val)),
            Step::NotReady(t, o) => Ok((t, o)),
        }
    }
}

/// Shorthand:
pub type AResult<M: MealyMachine> = Result<Step<M, M::Output, M::CalcResult>, M::Error>;

pub trait MealyMachine: Sized {
    type Input;
    type Output;
    type Error;
    type CalcResult;

    fn transition(self,
                  Self::Input)
                  -> AResult<Self>;

    fn and_then<M, F>(self, f: F) -> AndThen<Self, M, F>
        where M: MealyMachine<Input = Self::Input, Output = Self::Output, Error = Self::Error>,
              F: FnOnce(Self::CalcResult) -> M
    {
        AndThen::Machine1(self, f)
    }
}

pub enum AndThen<M1, M2, F>
    where M1: MealyMachine,
          M2: MealyMachine<Input = M1::Input, Output = M1::Output, Error = M1::Error>,
          F: FnOnce(M1::CalcResult) -> M2
{
    Machine1(M1, F),
    Machine2(M2),
}


impl<M1, M2, F> MealyMachine for AndThen<M1, M2, F>
    where M1: MealyMachine,
          M2: MealyMachine<Input = M1::Input, Output = M1::Output, Error = M1::Error>,
          F: FnOnce(M1::CalcResult) -> M2
{
    type Input = M1::Input;
    type Output = Option<M1::Output>;
    type Error = M1::Error;
    type CalcResult = M2::CalcResult;

    fn transition(self, input: Self::Input) -> AResult<Self> {
        match self {
            AndThen::Machine1(m1, f) => {
                match m1.transition(input)? {
                    Step::NotReady(new_m1, output) => {
                        Ok(Step::NotReady(AndThen::Machine1(new_m1, f), Some(output)))
                    }
                    Step::Done(cresult) => Ok(Step::NotReady(AndThen::Machine2(f(cresult)), None)),
                }
            }
            AndThen::Machine2(m2) => {
                match m2.transition(input)? {
                    Step::NotReady(new_m2, output) => {
                        Ok(Step::NotReady(AndThen::Machine2(new_m2), Some(output)))
                    }
                    Step::Done(cresult) => Ok(Step::Done(cresult)),
                }
            }
        }
    }
}

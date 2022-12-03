#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Round {
    Jeopardy,
    DoubleJeopardy,
    FinalJeopardy,
}

#[derive(Debug)]
pub struct JeopardyQuestion {
    pub question: String,
    pub category: String,
    pub round: Round,
    pub value: u32,
    pub answer: Option<String>,
}

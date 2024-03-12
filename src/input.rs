pub struct Handler {
    state: State,
}

pub enum Action {
    NextPage,
    PreviousPage,
    Leave,
    SetMark(char),
    FollowMark(char),
}

impl Handler {
    pub fn new() -> Self {
        Self {
            state: State::Normal,
        }
    }

    pub fn handle(&mut self, input: &str) -> Option<Action> {
        match self.state {
            State::Normal => match input {
                "ArrowRight" => Some(Action::NextPage),
                "ArrowLeft" => Some(Action::PreviousPage),
                "m" => {
                    self.state = State::SetMark;
                    None
                }
                "g" => {
                    self.state = State::FollowMark;
                    None
                }
                "Escape" => Some(Action::Leave),
                _ => None,
            },
            State::SetMark => {
                self.state = State::Normal;
                get_char(input).map(Action::SetMark)
            }
            State::FollowMark => {
                self.state = State::Normal;
                get_char(input).map(Action::FollowMark)
            }
        }
    }
}

fn get_char(input: &str) -> Option<char> {
    if input.len() == 1 && matches!(input.chars().next(), Some('a'..='z')) {
        return Some(input.chars().next().unwrap());
    }
    None
}

enum State {
    Normal,
    SetMark,
    FollowMark,
}

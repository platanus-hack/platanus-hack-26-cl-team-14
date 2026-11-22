use std::cmp::Reverse;

pub struct VisitOption<T> {
    /// Place and mode in the caller's representation.
    pub choice: T,
    /// All time the visit uses, including travel and the activity.
    pub total_minutes: u32,
    pub cost_clp: u32,
    /// The caller checks service hours, admission, and route availability.
    pub available: bool,
    /// Higher scores mean the person prefers this option.
    pub preference: i32,
}

/// Choose the feasible option with the highest preference score.
/// Keep the first option on a tie. Skip if no feasible option beats the skip score.
pub fn choose_visit<T>(
    options: &[VisitOption<T>],
    available_minutes: u32,
    budget_clp: u32,
    skip_preference: i32,
) -> Option<&T> {
    options
        .iter()
        .filter(|option| {
            option.available
                && option.total_minutes <= available_minutes
                && option.cost_clp <= budget_clp
                && option.preference > skip_preference
        })
        .min_by_key(|option| Reverse(option.preference))
        .map(|option| &option.choice)
}

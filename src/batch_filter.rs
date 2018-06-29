use super::repo::Repo;

#[derive(Debug, Clone, Copy)]
pub enum CombinationMode {
    Or,
    And,
    None,
}

pub trait BatchFilter {
    fn passes_through(&self, repo: &Repo) -> bool;

    fn combine<F>(self, mode: CombinationMode, other: F) -> Option<Box<dyn BatchFilter>>
    where
        Self: Sized + 'static,
        F: BatchFilter + 'static,
    {
        match mode {
            CombinationMode::Or => Some(Box::new(OrFilter(self, other))),
            CombinationMode::And => Some(Box::new(AndFilter(self, other))),
            CombinationMode::None => None,
        }
    }
}

impl<T> BatchFilter for Box<T>
where
    T: ?Sized + BatchFilter,
{
    fn passes_through(&self, repo: &Repo) -> bool {
        (**self).passes_through(repo)
    }
}

impl<T> BatchFilter for Option<T>
where
    T: BatchFilter,
{
    fn passes_through(&self, repo: &Repo) -> bool {
        if let Some(filter) = self {
            filter.passes_through(repo)
        } else {
            true
        }
    }

    fn combine<F>(self, mode: CombinationMode, other: F) -> Option<Box<dyn BatchFilter>>
    where
        Self: Sized + 'static,
        F: BatchFilter + 'static,
    {
        if let Some(filter) = self {
            match mode {
                CombinationMode::Or => Some(Box::new(OrFilter(filter, other))),
                CombinationMode::And => Some(Box::new(AndFilter(filter, other))),
                CombinationMode::None => None,
            }
        } else {
            Some(Box::new(other) as _)
        }
    }
}

pub struct VerbatimLabelFilter(pub String);

impl BatchFilter for VerbatimLabelFilter {
    fn passes_through(&self, repo: &Repo) -> bool {
        repo.labels.contains(&self.0)
    }
}

macro_rules! operator_filter {
    ($name:ident: x $operator:tt y) => {
        pub struct $name<F1, F2>(pub F1, pub F2);

        impl<F1, F2> BatchFilter for $name<F1, F2>
        where
            F1: BatchFilter,
            F2: BatchFilter,
        {
            fn passes_through(&self, repo: &Repo) -> bool {
                self.0.passes_through(repo) $operator self.1.passes_through(repo)
            }
        }
    }
}

operator_filter!(OrFilter: x || y);
operator_filter!(AndFilter: x && y);

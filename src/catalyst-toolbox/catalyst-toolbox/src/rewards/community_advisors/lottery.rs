use super::CommunityAdvisor;
use rand::Rng;
use std::collections::BTreeMap;

pub type TotalTickets = u64;
pub type TicketsDistribution = BTreeMap<CommunityAdvisor, TotalTickets>;
pub type CasWinnings = BTreeMap<CommunityAdvisor, TotalTickets>;

pub fn lottery_distribution<R: Rng>(
    mut distribution: TicketsDistribution,
    tickets_to_distribute: TotalTickets,
    rng: &mut R,
) -> (CasWinnings, TicketsDistribution) {
    let total_tickets = distribution.values().sum::<u64>() as usize;

    // Virtually create all tickets and choose the winning tickets using their index.
    let mut indexes =
        rand::seq::index::sample(rng, total_tickets, tickets_to_distribute as usize).into_vec();
    indexes.sort_unstable();
    let mut indexes = indexes.into_iter().peekable();

    // To avoid using too much memory, tickets are not actually created, and we iterate
    // the CAs to reconstruct the owner of each ticket.
    let mut winnings = CasWinnings::new();
    let mut cumulative_ticket_index = 0;

    // Consistent iteration is needed to get reproducible results. In this case,
    // it's ensured by the use of BTreeMap::iter()
    for (ca, n_tickets) in distribution.iter_mut() {
        let tickets_won = std::iter::from_fn(|| {
            indexes.next_if(|tkt| *tkt < (cumulative_ticket_index + *n_tickets) as usize)
        })
        .count();
        cumulative_ticket_index += *n_tickets;
        if tickets_won > 0 {
            winnings.insert(ca.clone(), tickets_won as u64);
        }
        *n_tickets -= tickets_won as u64;
    }
    (winnings, distribution)
}

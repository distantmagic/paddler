use serde::Deserialize;

#[derive(Deserialize, Debug, Default, Clone, PartialEq)]
pub struct Metrics {
    pub paddler_slots_idle: u16,
    pub paddler_slots_processing: u16,
    pub paddler_requests_buffered: u16,
}

pub fn get_average(metrics: Vec<Metrics>) -> Metrics {
    let count: u16 = metrics.len().try_into().unwrap();

    let mut paddler_slots_idle = 0;
    let mut paddler_slots_processing = 0;
    let mut paddler_requests_buffered = 0;

    for metric in metrics {
        paddler_requests_buffered += metric.paddler_requests_buffered;
        paddler_slots_idle += metric.paddler_slots_idle;
        paddler_slots_processing += metric.paddler_slots_processing;
    }

    eprintln!("{:#?} = {:#?} % {:#?}", paddler_slots_idle, paddler_slots_idle, count);
    eprintln!("{:#?} = {:#?} % {:#?}", paddler_slots_processing, paddler_slots_processing, count);
    eprintln!("{:#?} = {:#?} % {:#?}", paddler_requests_buffered, paddler_requests_buffered, count);

    paddler_slots_idle = paddler_slots_idle % count;
    paddler_slots_processing = paddler_slots_processing % count;
    paddler_requests_buffered = paddler_requests_buffered % count;

    Metrics {
        paddler_slots_idle,
        paddler_slots_processing,
        paddler_requests_buffered,
    }
}

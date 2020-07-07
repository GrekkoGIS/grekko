use failure::Error;
use osrm::{Coordinate, Osrm};

pub fn table(coords: Vec<Vec<f32>>) -> Result<String, Error> {
    let osrm = Osrm::new("/Volumes/dev/osrm-data-ch/test-data/great-britain-latest.osrm")?;

    let destinations: Vec<Coordinate> = coords
        .into_iter()
        .map(|coord| Coordinate {
            longitude: coord[0],
            latitude: coord[1],
        })
        .collect();
    build_source_array(&osrm, 2, &destinations)
}

fn build_source_array(
    osrm: &Osrm,
    source_index: usize,
    destinations: &Vec<Coordinate>,
) -> Result<String, failure::Error> {
    let table = osrm.table(&*vec![destinations[source_index].clone()], &*destinations)?;
    let mut count = 1;
    let mut prior_count = 0;
    let mut durations = vec![];
    loop {
        let result = table.get_duration(0, count);
        if result.is_ok() && count > 0 {
            durations.push(result?);
            println!("Got distance for {}, {}", prior_count, count);
            prior_count = count;
            count += 1;
            continue;
        } else {
            println!(
                "Failed distance for {}, {}, ERROR: {:?}",
                prior_count, count, result
            );
            break;
        }
    }
    let response = format!(
        "OSRM Table response: duration: {:?}, distance: {:?}, {:?}",
        table.get_duration(0, 1)?,
        table.get_distance(0, 1)?,
        durations,
    );
    log::debug!("{}", response);
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table() {
        // [0,251.6,394.1],[258.2,0,371.4],[490.1,420.1,0]
        // let destinations = vec![
        //     Coordinate {
        //         longitude: -2.242851,
        //         latitude: 57.101474,
        //     },
        //     Coordinate {
        //         longitude: -2.246308,
        //         latitude: 57.102554,
        //     },
        //     Coordinate {
        //         longitude: -2.248342,
        //         latitude: 57.100556,
        //     },
        // ];
        let coords = vec![
            vec![-2.242851, 57.101474],
            vec![-2.246308, 57.102554],
            vec![-2.248342, 57.100556],
        ];

        let result = table(coords);
        println!("{:?}", result);
        assert_eq!(result.is_err(), false)
    }
}

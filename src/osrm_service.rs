use failure::Error;
use osrm::{Coordinate, Osrm};

pub fn table(list: Vec<String>) -> Result<String, Error> {
    let osrm = Osrm::new("/Volumes/dev/osrm-data-ch/test-data/great-britain-latest.osrm")?;

    let sources = vec![
        Coordinate {
            longitude: -2.242851,
            latitude: 57.101474,
        },
        Coordinate {
            longitude: -2.246308,
            latitude: 57.102554,
        },
        Coordinate {
            longitude: -2.248342,
            latitude: 57.100556,
        },
    ];
    let destinations: Vec<Coordinate> = vec![
        // Coordinate {
        //     longitude: -2.255708,
        //     latitude: 57.084444,
        // },
        // Coordinate {
        //     longitude: -2.246308,
        //     latitude: 57.096656,
        // },
        // Coordinate {
        //     longitude: -2.258102,
        //     latitude: 57.100556,
        // },
        // Coordinate {
        //     longitude: -2.267513,
        //     latitude: 57.097085,
        // },
        // Coordinate {
        //     longitude: -2.252854,
        //     latitude: 57.099011,
        // },
        // Coordinate {
        //     longitude: -2.252854,
        //     latitude: 57.099011,
        // },
    ];
    let table = osrm.table(&*vec![sources[2].clone()], &*sources)?;
    let mut count = 1;
    let mut prior_count = 0;
    let mut durations = vec![];
    loop {
        let result = table.get_duration(0, count);
        if result.is_ok() && count > 0 {
            durations.push(result.unwrap());
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
        let result = table(vec![String::new()]);
        println!("{:?}", result);
        assert_eq!(result.is_err(), false)
    }
}

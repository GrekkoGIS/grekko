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
    let destinations = vec![
        Coordinate {
            longitude: -2.255708,
            latitude: 57.084444,
        },
        Coordinate {
            longitude: -2.246308,
            latitude: 57.096656,
        },
        Coordinate {
            longitude: -2.258102,
            latitude: 57.100556,
        },
        Coordinate {
            longitude: -2.267513,
            latitude: 57.097085,
        },
        Coordinate {
            longitude: -2.252854,
            latitude: 57.099011,
        },
    ];
    let table = osrm.table(&*sources, &*destinations)?;
    let response = format!(
        "OSRM Table response: duration: {:?}, distance: {:?}",
        table.get_duration(0, 5)?,
        table.get_distance(0, 5)?,
    );
    log::debug!("{}", response);
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table() {
        let result = table(vec![String::new()]);
        println!("{:?}", result);
        assert_eq!(result.is_err(), false)
    }
}

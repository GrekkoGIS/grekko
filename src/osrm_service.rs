use failure::Error;
use osrm::{Coordinate, Osrm};

pub fn table(list: Vec<String>) -> Result<String, Error> {
    let osrm = Osrm::new("/Volumes/dev/osrm-data-ch/test-data/great-britain-latest.osrm")?;

    let sources = vec![
        Coordinate {
            latitude: -2.242851,
            longitude: 57.101474,
        },
        Coordinate {
            latitude: -2.246308,
            longitude: 57.102554,
        },
        Coordinate {
            latitude: -2.248342,
            longitude: 57.100556,
        },
    ];
    let destinations = vec![
        Coordinate {
            latitude: -2.255708,
            longitude: 57.084444,
        },
        Coordinate {
            latitude: -2.246308,
            longitude: 57.096656,
        },
        Coordinate {
            latitude: -2.258102,
            longitude: 57.100556,
        },
        Coordinate {
            latitude: -2.267513,
            longitude: 57.097085,
        },
        Coordinate {
            latitude: -2.252854,
            longitude: 57.099011,
        },
    ];
    let table = osrm.table(&*sources, &*destinations)?;
    let response = format!(
        "OSRM Table response: duration: {:?}, distance: {:?}",
        table.get_duration(0, 0)?,
        table.get_distance(0, 0)?
    );
    log::debug!("{}", response);
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table() {
        let result = table(vec![String::new()]).unwrap();
        println!("{}", result);
        assert!(true)
    }
}

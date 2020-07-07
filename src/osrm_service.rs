use failure::Error;
use osrm::{Coordinate, Osrm};

pub fn get_matrix(coords: Vec<Vec<f32>>) -> Result<Vec<Vec<f32>>, Error> {
    let osrm = Osrm::new("/Volumes/dev/osrm-data-ch/test-data/great-britain-latest.osrm")?;

    let destinations: Vec<Coordinate> = coords
        .into_iter()
        .map(|coord| Coordinate {
            longitude: coord[0],
            latitude: coord[1],
        })
        .collect();
    let mut matrix: Vec<Vec<f32>> = vec![];
    for (index, _) in destinations.clone().iter().enumerate() {
        matrix.push(build_source_array(&osrm, index, &destinations)?);
    }
    Ok(matrix)
}

fn build_source_array(
    osrm: &Osrm,
    source_index: usize,
    destinations: &Vec<Coordinate>,
) -> Result<Vec<f32>, failure::Error> {
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
            log::trace!(
                "Failed distance for {}, {}, ERROR: {:?}",
                prior_count,
                count,
                result
            );
            break;
        }
    }

    log::debug!("Durations for index `{}` is {:?}", source_index, durations);
    println!("Durations for index `{}` is {:?}", source_index, durations);
    Ok(durations)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table() {
        // [[0,108,149.1],[937.2,0,41.1],[896.1,33.9,0]]

        // 0 108 149.1
        // 937.2 0 41.1
        // 896 33.9 0
        let coords = vec![
            vec![-2.242851, 57.101474],
            vec![-2.246308, 57.102554],
            vec![-2.248342, 57.100556],
        ];

        let result = get_matrix(coords);
        println!("{:?}", result);
        assert_eq!(result.is_err(), false)
    }
}

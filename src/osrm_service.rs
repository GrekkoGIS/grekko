use failure::Error;
use osrm::{Coordinate, Osrm, TableResponse};
use vrp_pragmatic::format::problem::Matrix;

pub fn get_matrix(coords: Vec<Vec<f32>>) -> Result<Matrix, Error> {
    let (trip_durations, trip_distances) = trip(coords)?;
    let matrix = Matrix {
        profile: Some("car".to_string()),
        timestamp: None,
        travel_times: trip_durations[0]
            .clone()
            .iter()
            .map(|val| *val as i64)
            .collect(),
        distances: trip_distances[0]
            .clone()
            .iter()
            .map(|val| *val as i64)
            .collect(),
        error_codes: None,
    };
    Ok(matrix)
}

pub fn trip(coords: Vec<Vec<f32>>) -> Result<(Vec<Vec<f32>>, Vec<Vec<f32>>), Error> {
    // let trip_durations = trip_durations(coords)?;

    let trip_durations = generic_trip(&coords, |osrm, index, destinations| {
        build_source_duration(&osrm, index, &destinations)
    })?;
    let trip_distances = generic_trip(&coords, |osrm, index, destinations| {
        build_source_distance(&osrm, index, &destinations)
    })?;
    Ok((trip_durations, trip_distances))
}

pub fn generic_trip<F>(coords: &Vec<Vec<f32>>, action: F) -> Result<Vec<Vec<f32>>, Error>
where
    F: Fn(&Osrm, usize, &Vec<Coordinate>) -> Result<Vec<f32>, failure::Error>,
{
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
        matrix.push(action(&osrm, index, &destinations)?);
    }
    Ok(matrix)
}

fn build_source_duration(
    osrm: &Osrm,
    source_index: usize,
    destinations: &Vec<Coordinate>,
) -> Result<Vec<f32>, failure::Error> {
    let table = osrm.table(&*vec![destinations[source_index].clone()], &*destinations)?;
    let mut count = 0;
    let mut prior_count = 0;
    let mut durations = vec![];
    loop {
        let result = table.get_duration(0, count);
        if result.is_ok() && count > 0 {
            let duration = result?;
            durations.push(duration);
            log::trace!(
                "Got distance for start `{}` and end `{}`, distance `{}` ",
                prior_count,
                count,
                duration
            );
            prior_count = count;
            count += 1;
            continue;
        } else if result.is_ok() && count == 0 {
            let duration = table.get_duration(0, 0)?;
            durations.push(duration);
            log::trace!(
                "Got duration for start `{}` and end `{}`, distance `{}` ",
                prior_count,
                count,
                duration
            );
            prior_count = count;
            count += 1;
            continue;
        } else {
            log::trace!(
                "Failed duration for {}, {}, Result: {:?}",
                prior_count,
                count,
                result
            );
            break;
        }
    }

    log::debug!("Durations for index `{}` is {:?}", source_index, durations);
    Ok(durations)
}

fn build_source_distance(
    osrm: &Osrm,
    source_index: usize,
    destinations: &Vec<Coordinate>,
) -> Result<Vec<f32>, failure::Error> {
    let table = osrm.table(&*vec![destinations[source_index].clone()], &*destinations)?;
    let mut count = 0;
    let mut prior_count = 0;
    let mut durations = vec![];
    loop {
        let result = table.get_distance(0, count);
        if result.is_ok() && count > 0 {
            let duration = result?;
            durations.push(duration);
            log::trace!(
                "Got distance for start `{}` and end `{}`, distance `{}` ",
                prior_count,
                count,
                duration
            );
            prior_count = count;
            count += 1;
            continue;
        } else if result.is_ok() && count == 0 {
            let duration = table.get_duration(0, 0)?;
            durations.push(duration);
            log::trace!(
                "Got distance for start `{}` and end `{}`, distance `{}` ",
                prior_count,
                count,
                duration
            );
            prior_count = count;
            count += 1;
            continue;
        } else {
            log::trace!(
                "Failed distance for {}, {}, Result: {:?}",
                prior_count,
                count,
                result
            );
            break;
        }
    }

    log::debug!("Durations for index `{}` is {:?}", source_index, durations);
    Ok(durations)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generic_trip_duration() {
        let expected: Vec<Vec<f32>> = vec![
            vec![0.0, 108.0, 149.1],
            vec![937.2, 0.0, 41.1],
            vec![896.1, 33.9, 0.0],
        ];
        let coords = vec![
            vec![-2.242851, 57.101474],
            vec![-2.246308, 57.102554],
            vec![-2.248342, 57.100556],
        ];

        let result = generic_trip(&coords, |osrm, index, destinations| {
            build_source_duration(&osrm, index, &destinations)
        });
        println!("{:?}", result);
        assert_eq!(result.is_err(), false);
        assert_eq!(expected, result.unwrap())
    }
}

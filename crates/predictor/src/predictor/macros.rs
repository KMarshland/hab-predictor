
/*
 * let a = result_or_return!(b)
 *
 * -------
 *
 * let a = match b {
 *      Ok(b) => {
 *          b
 *      },
 *      Err(why) => {
 *          return Err(why)
 *      }
 * }
 */
macro_rules! result_or_return {
    ($variable:expr) => {
        match $variable {
            Ok(value) => value,
            Err(why) => {
                return Err(why)
            }
        }
    };
}

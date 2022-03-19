use lambda_runtime::{LambdaEvent, Error};
use serde::{Serialize, Deserialize};
use z3::{Config, Context, Solver, Sort, SatResult, Model};
use z3::ast::{BV, Ast};

/// The tokio main method invoked upon spawning by Lambda. We register a
/// handler to deal with any incoming requests.
#[tokio::main]
async fn main( ) -> Result< (), Error > {
  let service = lambda_runtime::service_fn( handle );
  lambda_runtime::run( service ).await?;
  Ok( () )
}

/// Handle a single request incoming from a client.
/// 
/// We read the integer in the request, and find two non-negative integer values
/// that sum to it.
async fn handle( event : LambdaEvent< Request > ) -> Result< Output, Error > {
  let req = event.payload;
  // Note that `event.context.request_id` contains the Lambda-assigned ID

  if let Some( (x, y) ) = solve( req.val ) {
    Ok( Output { x, y } )
  } else {
    // If Z3 fails, just returns all zeros
    Ok( Output { x: 0, y: 0 } )
  }
}

/// The structure provided by the client as JSON.
/// We use Serde to parse the JSON into the structure. (with `Deserialize`)
#[derive(Deserialize)]
struct Request {
  val : u32
}

/// The structure provided to the client as JSON.
/// We use Serde to generate the JSON. (with `Serialize`)
#[derive(Serialize)]
struct Output {
  x : u32,
  y : u32
}

/// Call Z3 to finds 2 non-zero positive integers that sum to the provided
/// `val`. (Note that this fails when `val < 2`)
fn solve( val : u32 ) -> Option< (u32, u32) > {
  let cfg = Config::new();
  let ctx = Context::new(&cfg);

  let solver = Solver::new(&ctx);

  let x = z3::ast::BV::new_const( &ctx, "x", 32 );
  let y = z3::ast::BV::new_const( &ctx, "y", 32 );

  let n_val = z3::ast::BV::from_u64( &ctx, val as u64, 32 );
  let n0 = z3::ast::BV::from_u64( &ctx, 0, 32 );

  solver.assert( &x.bvugt( &n0 ) );
  solver.assert( &y.bvugt( &n0 ) );

  solver.assert( &x.bvult( &n_val ) );
  solver.assert( &y.bvult( &n_val ) );

  solver.assert( &x.bvadd( &y )._eq( &n_val ) );

  /// Helper. Evaluates a bitvector to a `u32` under the given model.
  fn bv_value( model: &Model, x: &BV ) -> Option< u32 > {
    let v = model.eval( x, true )?;
    let q = v.as_u64( )?;
    Some( q as u32 )
  }

  match solver.check( ) {
    SatResult::Unsat => None,
    SatResult::Unknown => None,
    SatResult::Sat => {
      if let Some( model ) = solver.get_model( ) {
        let x_val = bv_value( &model, &x ).unwrap_or( 0 );
        let y_val = bv_value( &model, &y ).unwrap_or( 0 );

        Some( (x_val, y_val) )
      } else {
        None
      }
    }
  }
}

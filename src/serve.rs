use crate::{
    bfs::extract_subgraph_by_bfs,
    echart::EchartGraph,
    gfa::{gfa_to_graph, GFAGraph},
    gml::{GMLGraph, GMLObject},
    layout::Layout,
    vcf::{parse_vcf_file, BubbleVariant},
};
use actix_files as fs;
use actix_web::{
    error::{ErrorInternalServerError, ErrorNotFound},
    get, web, App, HttpServer, Responder, Result as ActixResult,
};
use anyhow::Result;
use std::collections::HashMap;
use std::{str::FromStr, sync::Arc};

pub struct AppState {
    gfa: Arc<GFAGraph>,
    variants: Arc<HashMap<String, BubbleVariant>>,
}

#[get("/api/variants")]
async fn get_variants(data: web::Data<AppState>) -> impl Responder {
    let variant_ids: Vec<_> = data.variants.keys().cloned().collect();
    web::Json(variant_ids)
}

#[get("/api/layout/{variant_id}")]
async fn get_layout(
    path: web::Path<String>,
    data: web::Data<AppState>,
) -> ActixResult<impl Responder> {
    let variant_id = path.into_inner();
    let variant = data
        .variants
        .get(&variant_id)
        .ok_or_else(|| ErrorNotFound("Variant not found"))?;

    let sub_graph = extract_subgraph_by_bfs(variant, &data.gfa)
        .map_err(|e| ErrorInternalServerError(e.to_string()))?;

    let gml_content = sub_graph.to_gml_string();
    let origin_g =
        GMLObject::from_str(&gml_content).map_err(|e| ErrorInternalServerError(e.to_string()))?;
    let origin_g =
        GMLGraph::from_gml(origin_g).map_err(|e| ErrorInternalServerError(e.to_string()))?;

    let layout = Layout::new().map_err(|e| ErrorInternalServerError(e.to_string()))?;
    let res = layout
        .run(&gml_content)
        .map_err(|e| ErrorInternalServerError(e.to_string()))?;

    let layout_graph =
        GMLObject::from_str(&res).map_err(|e| ErrorInternalServerError(e.to_string()))?;
    let layout_graph =
        GMLGraph::from_gml(layout_graph).map_err(|e| ErrorInternalServerError(e.to_string()))?;

    let echart_graph = EchartGraph::from_gml_anno(layout_graph, origin_g)
        .map_err(|e| ErrorInternalServerError(e.to_string()))?;

    Ok(web::Json(echart_graph))
}

pub async fn serve(gfa_path: &str, vcf_path: &str, port: u16) -> Result<()> {
    // Read GFA file and VCF file
    let gfa = Arc::new(gfa_to_graph(gfa_path)?);

    let variants: HashMap<_, _> = parse_vcf_file(vcf_path)?
        .into_iter()
        .map(|v| (v.id.clone(), v))
        .collect();
    let variants = Arc::new(variants);

    // Prepare app state
    let app_state = web::Data::new(AppState { gfa, variants });

    println!("Server running at http://localhost:{}", port);

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(get_variants)
            .service(get_layout)
            .service(fs::Files::new("/", "./static").index_file("index.html"))
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await?;

    Ok(())
}

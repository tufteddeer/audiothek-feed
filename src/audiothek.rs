use anyhow::Result;
use atom_syndication::{Content, Entry, Feed, Link, Text};
use chrono::DateTime;
use graphql_client::{GraphQLQuery, Response};
use program_set::{ProgramSetProgramSetItemsNodes, ProgramSetProgramSetItemsNodesAudios};
use serde::de::DeserializeOwned;

use self::program_metadata::ProgramMetadataProgramSet;

const GRAPHQL_ENDPOINT: &str = "https://api.ardaudiothek.de/graphql";

type Datetime = String;
#[allow(clippy::upper_case_acronyms)]
type URL = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "query.graphql",
    response_derives = "Debug"
)]
pub struct ProgramSet;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "metadata_query.graphql",
    response_derives = "Debug"
)]
/// Get metadata, without episodes
pub struct ProgramMetadata;

/// Execute GraphQL query with the given variables
async fn graphql_query<X, V, T>(variables: V) -> Result<T>
where
    X: graphql_client::GraphQLQuery<Variables = V>,
    V: serde::Serialize,
    T: DeserializeOwned,
{
    let request_body = X::build_query(variables);

    let client = reqwest::Client::new();
    let res = client
        .post(GRAPHQL_ENDPOINT)
        .json(&request_body)
        .send()
        .await?;
    let response_body: Response<T> = res.json().await?;

    response_body
        .data
        .ok_or(anyhow::anyhow!("Failed to fetch program metadata"))
}

pub async fn fetch_metadata(
    variables: program_metadata::Variables,
) -> Result<ProgramMetadataProgramSet> {
    let response_body = graphql_query::<
        ProgramMetadata,
        program_metadata::Variables,
        program_metadata::ResponseData,
    >(variables)
    .await?;

    let program_meta = response_body
        .program_set
        .ok_or(anyhow::anyhow!("Failed to fetch program metadata"))?;

    Ok(program_meta)
}
/// Load the data from the Audiothek, then build the atom feed
pub async fn fetch_feed(variables: program_set::Variables) -> Result<Feed> {
    let response_body =
        graphql_query::<ProgramSet, program_set::Variables, program_set::ResponseData>(variables)
            .await?;

    let program_set = response_body
        .program_set
        .ok_or(anyhow::anyhow!("Failed to fetch program"))?;

    let entries: Vec<Entry> = program_set
        .items
        .nodes
        .iter()
        .map(node_to_episode)
        .collect();

    let mut atom_feed = Feed {
        title: program_set.title.into(),
        entries,
        id: program_set.id,
        ..Default::default()
    };

    if let Some(Ok(updated)) = program_set
        .last_item_modified
        .map(|date| DateTime::parse_from_rfc3339(&date))
    {
        atom_feed.updated = updated;
    }

    atom_feed.logo = program_set
        .image
        .as_ref()
        .and_then(|image| image.url.clone())
        .map(|url| image_url(&url, 512));

    atom_feed.icon = program_set
        .image
        .as_ref()
        .and_then(|image| image.url.clone())
        .map(|url| image_url(&url, 256));

    Ok(atom_feed)
}

/// Fill the {width} placeholder of an image url
pub fn image_url(url: &str, width: u32) -> String {
    url.replace("{width}", format!("{width}").as_str())
}

fn audio_to_link(audio: &ProgramSetProgramSetItemsNodesAudios) -> Link {
    Link {
        href: audio.url.clone(),
        rel: "enclosure".to_string(),
        mime_type: Some(audio.mime_type.clone()),
        title: audio.title.clone(),
        ..Default::default()
    }
}

fn node_to_episode(item: &ProgramSetProgramSetItemsNodes) -> Entry {
    let links = item
        .audios
        .as_ref()
        .map(|audios| audios.iter().map(audio_to_link))
        .unwrap()
        .collect();

    let summary = item.summary.clone();
    let published = DateTime::parse_from_rfc3339(&item.publish_date).ok();

    let content = Content {
        content_type: Some("text".to_string()),
        value: item.summary.clone(),
        ..Content::default()
    };

    let mut entry = Entry {
        id: item.id.clone(),
        title: Text::plain(item.title.clone()),
        summary: summary.map(Text::plain),
        content: Some(content),
        links,
        published,
        ..Entry::default()
    };

    if let Some(published) = published {
        entry.updated = published;
    }

    entry
}

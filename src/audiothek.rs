use core::fmt;

use anyhow::Result;
use atom_syndication::{Content, Entry, Feed, Link, LinkBuilder, Text};
use chrono::DateTime;
use graphql_client::{GraphQLQuery, Response};
use serde::de::DeserializeOwned;

use self::{
    program_metadata::ProgramMetadataProgramSet,
    program_set::{
        ProgramSetProgramSet, ProgramSetProgramSetItemsEdgesNode,
        ProgramSetProgramSetItemsEdgesNodeAudios,
    },
};

const GRAPHQL_ENDPOINT: &str = "https://api.ardaudiothek.de/graphql";

type Datetime = String;
#[allow(clippy::upper_case_acronyms)]
type URL = String;
type Cursor = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "query.graphql",
    response_derives = "Debug",
    variables_derives = "Debug"
)]
pub struct ProgramSet;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "metadata_query.graphql",
    response_derives = "Debug",
    variables_derives = "Debug"
)]
/// Get metadata, without episodes
pub struct ProgramMetadata;

/// Execute GraphQL query with the given variables
#[tracing::instrument(name = "graphql_query")]
async fn graphql_query<X, V, T>(variables: V) -> Result<T>
where
    X: graphql_client::GraphQLQuery<Variables = V>,
    V: serde::Serialize + fmt::Debug,
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

    if let Some(errors) = response_body.errors {
        for err in errors {
            log::error!("{err}");
        }
    }

    response_body
        .data
        .ok_or(anyhow::anyhow!("Failed to run GraphQL query"))
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
        .ok_or(anyhow::anyhow!("No result"))?;

    Ok(program_meta)
}

#[tracing::instrument(name = "create_feed", skip_all)]
fn create_feed(program_set: ProgramSetProgramSet, host: String) -> Result<Feed> {
    let entries: Vec<Entry> = program_set
        .items
        .edges
        .iter()
        .map(|edge| &edge.node)
        .map(node_to_episode)
        .collect();

    let mut atom_feed = Feed {
        title: program_set.title.into(),
        entries,
        id: program_set.id.clone(),
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

    let cursor = &program_set
        .items
        .edges
        .first()
        .as_ref()
        .and_then(|last| last.cursor.clone());

    if let Some(next_page_cursor) = cursor {
        let url = format!(
            "http://{host}/feed/{}?cursor={next_page_cursor}",
            program_set.id
        );

        let next_page = LinkBuilder::default()
            .rel("next".to_string())
            .href(url)
            .build();

        atom_feed.links.push(next_page);
    }

    Ok(atom_feed)
}

/// Load the data from the Audiothek, then build the atom feed
#[tracing::instrument(name = "fetch_feed")]
pub async fn fetch_feed(variables: program_set::Variables, host: String) -> Result<Feed> {
    let response_body =
        graphql_query::<ProgramSet, program_set::Variables, program_set::ResponseData>(variables)
            .await?;

    let program_set = response_body
        .program_set
        .ok_or(anyhow::anyhow!("No result"))?;

    create_feed(program_set, host)
}

/// Fill the {width} placeholder of an image url
pub fn image_url(url: &str, width: u32) -> String {
    url.replace("{width}", format!("{width}").as_str())
}

fn audio_to_link(audio: &ProgramSetProgramSetItemsEdgesNodeAudios) -> Link {
    Link {
        href: audio.url.clone(),
        rel: "enclosure".to_string(),
        mime_type: Some(audio.mime_type.clone()),
        title: audio.title.clone(),
        ..Default::default()
    }
}

fn node_to_episode(item: &ProgramSetProgramSetItemsEdgesNode) -> Entry {
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

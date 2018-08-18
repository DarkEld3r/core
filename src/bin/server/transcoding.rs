use actix_web;
use actix_web::dev::Handler;
use actix_web::error;
use actix_web::AsyncResponder;
use actix_web::FromRequest;
use actix_web::HttpRequest;
use actix_web::Path;
use actix_web::State;

use forte_core::models::Song;

use server::graphql::AppState;
use server::stream::RangeStream;
use server::transcoder::{TranscodeMessage, TranscodeTarget};

use diesel;

use uuid::Uuid;

use futures::future;
use futures::Future;

use lru_disk_cache::ReadSeek;

use std::io;
use std::io::SeekFrom;
use std::path::PathBuf;

pub struct TranscodedSongHandler {
    transcode_target: TranscodeTarget,
}

impl TranscodedSongHandler {
    pub fn new(target: TranscodeTarget) -> TranscodedSongHandler {
        TranscodedSongHandler {
            transcode_target: target,
        }
    }
}

impl Handler<AppState> for TranscodedSongHandler {
    type Result = actix_web::FutureResponse<RangeStream<Box<ReadSeek>>>;

    fn handle(&mut self, req: HttpRequest<AppState>) -> Self::Result {
        let state: State<AppState> = State::from_request(&req, &());
        let song_id_path: Path<Uuid> =
            Path::from_request(&req, &()).expect("song id path parameter missing");
        let song_id = song_id_path.into_inner();

        future::done(build_transcode_message(
            song_id,
            self.transcode_target.clone(),
            &state,
        )).and_then(move |transcode_msg| {
            state
                .transcoder
                .send(transcode_msg)
                .map_err(error::ErrorInternalServerError)
        })
            .and_then(|result| {
                result.map_err(|e| error::ErrorInternalServerError(e.description().to_string()))
            })
            .and_then(|mut reader| {
                get_size(&mut reader)
                    .map(|size| (reader, size))
                    .map_err(error::ErrorInternalServerError)
            })
            .map(|(reader, size)| RangeStream::new(reader, size))
            .responder()
    }
}

fn build_transcode_message(
    song_id: Uuid,
    transcode_target: TranscodeTarget,
    state: &State<AppState>,
) -> actix_web::Result<TranscodeMessage<PathBuf>> {
    let context = state
        .build_context()
        .map_err(error::ErrorInternalServerError)?;

    let song = Song::from_id(&context, &song_id.into()).map_err(convert_diesel_err)?;

    let song_path: PathBuf = song.path.into();

    Ok(TranscodeMessage::new(
        song_path,
        song_id.to_string(),
        transcode_target,
    ))
}

fn get_size<R: ReadSeek>(reader: &mut R) -> io::Result<u64> {
    let size = reader.seek(SeekFrom::End(0))?;
    reader.seek(SeekFrom::Start(0))?;

    Ok(size)
}

fn convert_diesel_err(err: diesel::result::Error) -> actix_web::Error {
    match err {
        diesel::result::Error::NotFound => error::ErrorNotFound(err),
        _ => error::ErrorInternalServerError(err),
    }
}

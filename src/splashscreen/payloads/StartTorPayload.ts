export type StartTorPayload = {
    progress: number,
    message: string
}
/*
struct TorStartupErrorPayload {
    message: Option<String>,
    error_code: Option<i32>,
    logs: Option<Vec<String>>,
}
*/

export type StartTorErrorPayload = {
    message: string,
    error_code: number,
    logs: string[]
}
use jupyter_protocol::{ExecutionState, JupyterMessage, JupyterMessageContent};
use serde_json::json;

use super::{KernelEvent, Reply};

pub(super) fn kernel_event(message: JupyterMessage) -> Option<KernelEvent> {
    let request = message.parent_header?.msg_id;
    let reply = match message.content {
        JupyterMessageContent::StreamContent(stream) => Reply::Output(json!({
            "output_type": "stream",
            "name": stream.name,
            "text": stream.text,
        })),
        JupyterMessageContent::DisplayData(display) => Reply::Output(json!({
            "output_type": "display_data",
            "data": display.data,
            "metadata": display.metadata,
        })),
        JupyterMessageContent::ExecuteResult(result) => Reply::Output(json!({
            "output_type": "execute_result",
            "execution_count": result.execution_count,
            "data": result.data,
            "metadata": result.metadata,
        })),
        JupyterMessageContent::ErrorOutput(error) => Reply::Output(json!({
            "output_type": "error",
            "ename": error.ename,
            "evalue": error.evalue,
            "traceback": error.traceback,
        })),
        JupyterMessageContent::ClearOutput(clear) => Reply::Clear { wait: clear.wait },
        JupyterMessageContent::ExecuteInput(input) => {
            Reply::ExecutionCount(input.execution_count.value() as u64)
        }
        JupyterMessageContent::Status(status) if status.execution_state == ExecutionState::Idle => {
            Reply::Idle
        }
        _ => return None,
    };
    Some(KernelEvent::Reply { request, reply })
}

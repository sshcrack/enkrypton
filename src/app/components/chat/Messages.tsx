import { useContext } from 'react'
import RenderIfVisible from 'react-render-if-visible'
import { ChatContext } from './ChatProvider'
import { MessageBox } from 'react-chat-elements'

export type MessagesProps = {}

const ESTIMATED_ITEM_HEIGHT = 100
export default function Messages({ }: MessagesProps) {
    const { messages, additional } = useContext(ChatContext)

    return <>
        {messages.map(({ msg, self_sent, date }, i) => {
            let { failed, sending } = additional.get(date) ?? { failed: false, sending: false }

            const msgComp = <MessageBox
                position={self_sent ? "right" : "left"}
                type={'text'}
                key={i}
                date={date}
                focus={false}
                forwarded={false}
                id={date}
                notch={failed}
                removeButton={false}
                replyButton={false}
                retracted={failed}
                statusTitle={failed ? "Failed to send" : undefined}
                status={sending || failed ? "waiting" : "received"}
                text={msg}
                title={failed ? "Failed to send" : (self_sent ? "You" : "Other")}
                titleColor={failed ? "red" : 'white'}
            />

            return <RenderIfVisible defaultHeight={ESTIMATED_ITEM_HEIGHT}>
                {msgComp}
            </RenderIfVisible>
        })}
        </>
}
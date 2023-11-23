import { useContext } from 'react'
import RenderIfVisible from 'react-render-if-visible'
import { ChatContext } from './ChatProvider'
import { MessageBox } from 'react-chat-elements'
import { Spinner } from '@chakra-ui/react'

export type MessagesProps = {}

const ESTIMATED_ITEM_HEIGHT = 100
export default function Messages({ }: MessagesProps) {
    const { client } = useContext(ChatContext)
    if(!client)
        return <Spinner />

    return <>
        {client.messages().map(({ msg, self_sent, date, status }, i) => {
            let statusMsg: 'waiting' | 'sent' | 'received' | 'read' = "waiting";
            switch (status) {
                case "Failed":
                    statusMsg = "sent"
                    break;

                case "Sending":
                    statusMsg = "waiting"
                    break;

                case "Sent":
                    statusMsg = "sent"
                    break;

                case "Success":
                    statusMsg = "received";
                    break;
            }

            const failed = status === "Failed"
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
                statusTitle={status ? "Failed to send" : undefined}
                status={statusMsg}
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
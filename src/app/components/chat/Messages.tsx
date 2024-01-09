import { useContext } from 'react'
import RenderIfVisible from 'react-render-if-visible'
import { ChatContext } from './ChatProvider'
import { MessageBox } from 'react-chat-elements'
import { Flex, Spinner, Text } from '@chakra-ui/react'

export type MessagesProps = {}

/**
 * Estimated height for each message .
 */
const ESTIMATED_ITEM_HEIGHT = 100
/**
 * Displays the messages of the currently active client (uses ChatContext).
 * @param props No Props here
 */
export default function Messages({ }: MessagesProps) {
    const { client } = useContext(ChatContext)
    if(!client)
        return <Spinner />

    const msg = client.messages()
    return <>
        {msg.map(({ msg, self_sent, date, status }, i) => {
            // Parsing backend status to message status
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

            // Rendering the message only if it is visible
            return <RenderIfVisible defaultHeight={ESTIMATED_ITEM_HEIGHT}>
                {msgComp}
            </RenderIfVisible>
        })}
        {msg.length === 0 && <Flex w='100%' h='100%' pt='10' justifyContent='center'><Text>No Messages sent yet.</Text></Flex>}
        </>
}
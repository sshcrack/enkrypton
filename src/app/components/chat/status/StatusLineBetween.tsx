import { Flex } from '@chakra-ui/react';
import styles from "./status.module.scss"

export type StatusLineBetweenProps = {
    /**
     * Whether the line should be displayed green (so done) or not.
     */
    isDone: boolean,
    /**
     * Whether the line should be displayed as loading.
     */
    animate: boolean,
    /**
     * Whether the line should be displayed red (so failed) or not.
     */
    isFailed: boolean
}

/**
 * The status line between two steps for connecting to a client.
 * @param props For more Doc look at StatusLineBetweenProps
 */
export default function StatusLineBetween({ animate, isDone, isFailed }: StatusLineBetweenProps) {
    return <Flex
        h='10px'
        rounded='xl'
        className={`${styles.lineBetween} ${isFailed ? styles.failed : isDone ? styles.done : ""}`}
    ><Flex className={`${styles.loader} ${animate && !isDone ? styles.animate : ""}`}></Flex></Flex>
}
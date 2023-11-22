import { Flex } from '@chakra-ui/react';
import styles from "./status.module.scss"

export type StatusLineBetweenProps = {
    isDone: boolean,
    animate: boolean,
    isFailed: boolean
}

export default function StatusLineBetween({ animate, isDone, isFailed }: StatusLineBetweenProps) {
    return <Flex
        h='10px'
        rounded='xl'
        className={`${styles.lineBetween} ${isFailed ? styles.failed : isDone ? styles.done : ""}`}
    ><Flex className={`${styles.loader} ${animate && !isDone ? styles.animate : ""}`}></Flex></Flex>
}
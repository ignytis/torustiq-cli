#ifndef _TORUSTIQ_CLI_COMMON_COLLECTIONS_H_
#define _TORUSTIQ_CLI_COMMON_COLLECTIONS_H_

// TODO: limit the queue size?

// Credits:
// https://www.geeksforgeeks.org/dsa/implement-thread-safe-queue-in-c/

// C++ implementation of the above approach
#include <condition_variable>
#include <iostream>
#include <mutex>
#include <queue>

namespace TorustiqCli {
namespace Common {
namespace Collections {

// Thread-safe queue
template <typename T>
class TSQueue {
   private:
    // Underlying queue
    std::queue<T> m_queue;

    // mutex for thread synchronization
    std::mutex m_mutex;

    // Condition variable for signaling
    std::condition_variable m_cond;

   public:
    // Pushes an element to the queue
    void push(T item) {
        // Acquire lock
        std::unique_lock<std::mutex> lock(m_mutex);

        // Add item
        m_queue.push(item);

        // Notify one thread that
        // is waiting
        m_cond.notify_one();
    }

    // Pops an element off the queue
    T pop() {
        // acquire lock
        std::unique_lock<std::mutex> lock(m_mutex);

        // wait until queue is not empty
        m_cond.wait(lock, [this]() { return !m_queue.empty(); });

        // retrieve item
        T item = m_queue.front();
        m_queue.pop();

        // return item
        return item;
    }
};

}  // namespace Collections
}  // namespace Common
}  // namespace TorustiqCli

#endif

//
//  IronJumpAppDelegate.h
//  IronJump
//
//  Created by Filip Kunc on 8/1/10.
//  For license see LICENSE.TXT
//


#import <UIKit/UIKit.h>

@class EAGLView;

@interface IronJumpAppDelegate : NSObject <UIApplicationDelegate> 
{
    UIWindow *window;
    EAGLView *glView;
}

@property (nonatomic, retain) IBOutlet UIWindow *window;
@property (nonatomic, retain) IBOutlet EAGLView *glView;

@end


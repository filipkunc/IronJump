//
//  IronJumpAppDelegate.h
//  IronJump
//
//  Created by Filip Kunc on 8/1/10.
//  Copyright (c) 2010 Filip Kunc. All rights reserved.
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

